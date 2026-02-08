//! Cipher backend selection and dispatch.
//!
//! Provides a unified interface for multiple cipher backends.

use crate::core::config::Config;
use crate::error::{CipherError, Result};
#[cfg(any(test, feature = "aws", feature = "gcp"))]
use crate::error::{ConfigError, Error};
#[cfg(any(test, feature = "aws", feature = "gcp"))]
use serde::{Deserialize, Serialize};
use tracing::debug;

#[cfg(any(test, feature = "aws", feature = "gcp"))]
const WRAPPED_CIPHERTEXT_VERSION: &str = "dugout-ciphertext-envelope-v1";

#[cfg(any(test, feature = "aws", feature = "gcp"))]
#[derive(Debug, Serialize, Deserialize)]
struct WrappedCiphertext {
    version: String,
    ciphertext: String,
}

#[cfg(any(test, feature = "aws", feature = "gcp"))]
fn wrap_for_recipients(ciphertext: &str, recipients: &[String]) -> Result<String> {
    use super::Cipher;

    let age_recipients: Result<Vec<_>> = recipients
        .iter()
        .map(|recipient| super::parse_recipient(recipient))
        .collect();
    let age_recipients = age_recipients?;

    if age_recipients.is_empty() {
        return Err(ConfigError::NoRecipients.into());
    }

    let ciphertext = super::Age.encrypt(ciphertext, &age_recipients)?;
    let envelope = WrappedCiphertext {
        version: WRAPPED_CIPHERTEXT_VERSION.to_string(),
        ciphertext,
    };

    serde_json::to_string(&envelope).map_err(Error::from)
}

#[cfg(any(test, feature = "aws", feature = "gcp"))]
fn unwrap_for_identity(payload: &str, identity: &age::x25519::Identity) -> Result<Option<String>> {
    use super::Cipher;

    let envelope = match serde_json::from_str::<WrappedCiphertext>(payload) {
        Ok(envelope) => envelope,
        Err(_) => {
            // Reject malformed envelope-like JSON payloads rather than attempting KMS decryption.
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(payload) {
                if value.get("version").is_some()
                    || value.get("ciphertext").is_some()
                    || value.get("wrapped_kms_ciphertext").is_some()
                {
                    return Err(CipherError::DecryptionFailed(
                        "invalid ciphertext envelope format".to_string(),
                    )
                    .into());
                }
            }
            return Ok(None);
        }
    };

    if envelope.version != WRAPPED_CIPHERTEXT_VERSION {
        return Err(CipherError::DecryptionFailed(format!(
            "unsupported ciphertext envelope version: {}",
            envelope.version
        ))
        .into());
    }

    let ciphertext = super::Age.decrypt(&envelope.ciphertext, identity)?;
    Ok(Some(ciphertext))
}

/// Vault cipher selection and dispatch.
///
/// Wraps the different cipher implementations and provides
/// dynamic dispatch based on configuration. Supports hybrid mode
/// where secrets are encrypted for both age (developers) and KMS (production).
#[derive(Debug)]
pub enum CipherBackend {
    /// Age encryption (default, always available)
    Age,

    /// Hybrid: age + KMS encryption
    #[cfg(any(test, feature = "test-kms"))]
    Hybrid {
        kms: super::kms::MockKms,
        provider: super::kms::KmsProvider,
    },

    #[cfg(feature = "aws")]
    /// AWS KMS encryption (legacy single-backend mode)
    AwsKms { key_id: String },

    #[cfg(feature = "aws")]
    /// Hybrid: age + AWS KMS
    HybridAws { key_id: String },

    #[cfg(feature = "gcp")]
    /// GCP KMS encryption (legacy single-backend mode)
    GcpKms { resource: String },

    #[cfg(feature = "gcp")]
    /// Hybrid: age + GCP KMS
    HybridGcp { resource: String },

    #[cfg(feature = "gpg")]
    /// GPG encryption
    Gpg,
}

#[allow(dead_code)] // Methods used by vault
impl CipherBackend {
    /// Create a cipher backend from configuration
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Unknown cipher type specified
    /// - Required configuration fields are missing
    /// - Cipher type requires a feature that wasn't compiled in
    pub fn from_config(config: &Config) -> Result<Self> {
        let cipher_type = config.dugout.cipher.as_deref().unwrap_or("age");

        debug!(cipher = %cipher_type, has_kms = config.has_kms(), "creating cipher backend");

        // Check for hybrid mode: age cipher + [kms] section
        if cipher_type == "age" {
            if let Some(kms_key) = config.kms_key() {
                return Self::hybrid_from_key(kms_key);
            }
        }

        match cipher_type {
            "age" => Ok(Self::Age),

            "aws-kms" => {
                #[cfg(feature = "aws")]
                {
                    let key_id =
                        config
                            .dugout
                            .kms_key_id
                            .clone()
                            .ok_or(ConfigError::MissingField {
                                field: "kms_key_id",
                            })?;
                    Ok(Self::AwsKms { key_id })
                }
                #[cfg(not(feature = "aws"))]
                {
                    Err(CipherError::EncryptionFailed(
                        "AWS KMS support not compiled. Rebuild with: cargo install dugout --features aws".to_string()
                    ).into())
                }
            }

            "gcp-kms" => {
                #[cfg(feature = "gcp")]
                {
                    let resource =
                        config
                            .dugout
                            .gcp_resource
                            .clone()
                            .ok_or(ConfigError::MissingField {
                                field: "gcp_resource",
                            })?;
                    Ok(Self::GcpKms { resource })
                }
                #[cfg(not(feature = "gcp"))]
                {
                    Err(CipherError::EncryptionFailed(
                        "GCP KMS support not compiled. Rebuild with: cargo install dugout --features gcp".to_string()
                    ).into())
                }
            }

            "gpg" => {
                #[cfg(feature = "gpg")]
                {
                    Ok(Self::Gpg)
                }
                #[cfg(not(feature = "gpg"))]
                {
                    Err(CipherError::EncryptionFailed(
                        "GPG support not compiled. Rebuild with: cargo install dugout --features gpg".to_string()
                    ).into())
                }
            }

            other => Err(CipherError::EncryptionFailed(format!(
                "unknown cipher type: {}. Supported: age, aws-kms, gcp-kms, gpg",
                other
            ))
            .into()),
        }
    }

    /// Create a hybrid backend from a KMS key identifier.
    fn hybrid_from_key(kms_key: &str) -> Result<Self> {
        use super::kms::KmsProvider;

        let provider = KmsProvider::detect(kms_key).ok_or_else(|| {
            CipherError::EncryptionFailed(format!(
                "unrecognized KMS key format: {}. Expected AWS ARN (arn:aws:kms:...) or GCP resource (projects/...)",
                kms_key
            ))
        })?;

        match provider {
            #[cfg(any(test, feature = "test-kms"))]
            KmsProvider::Aws | KmsProvider::Gcp => Ok(Self::Hybrid {
                kms: super::kms::MockKms,
                provider,
            }),

            #[cfg(all(feature = "aws", not(test)))]
            KmsProvider::Aws => Ok(Self::HybridAws {
                key_id: kms_key.to_string(),
            }),

            #[cfg(all(feature = "gcp", not(test)))]
            KmsProvider::Gcp => Ok(Self::HybridGcp {
                resource: kms_key.to_string(),
            }),

            #[cfg(all(not(feature = "aws"), not(test)))]
            KmsProvider::Aws => Err(CipherError::EncryptionFailed(
                "AWS KMS support not compiled. Rebuild with: cargo install dugout --features aws"
                    .to_string(),
            )
            .into()),

            #[cfg(all(not(feature = "gcp"), not(test)))]
            KmsProvider::Gcp => Err(CipherError::EncryptionFailed(
                "GCP KMS support not compiled. Rebuild with: cargo install dugout --features gcp"
                    .to_string(),
            )
            .into()),
        }
    }

    /// Encrypt plaintext with age for all recipients.
    fn encrypt_age(plaintext: &str, recipients: &[String]) -> Result<String> {
        use super::Cipher;
        let age_recipients: Result<Vec<_>> = recipients
            .iter()
            .map(|r| super::parse_recipient(r))
            .collect();
        super::Age.encrypt(plaintext, &age_recipients?)
    }

    /// Encrypt plaintext for the given recipients.
    ///
    /// In hybrid mode, produces an envelope with both age and KMS ciphertext.
    /// In age-only mode, produces raw age ciphertext (no envelope).
    ///
    /// # Errors
    ///
    /// Returns `CipherError` if encryption fails.
    pub fn encrypt(&self, plaintext: &str, recipients: &[String]) -> Result<String> {
        #[allow(unused_imports)]
        use crate::core::cipher::kms::Envelope;

        match self {
            Self::Age => Self::encrypt_age(plaintext, recipients),

            #[cfg(any(test, feature = "test-kms"))]
            Self::Hybrid { kms, provider } => {
                use super::kms::KmsBackend;
                let age_ct = Self::encrypt_age(plaintext, recipients)?;
                let kms_ct = kms.encrypt(plaintext)?;
                Envelope::new(age_ct, Some(kms_ct), Some(provider)).seal()
            }

            #[cfg(feature = "aws")]
            Self::AwsKms { key_id } => {
                use super::Cipher;
                let kms = super::aws::AwsKms::new(key_id.clone());
                let kms_ciphertext = kms.encrypt(plaintext, &[])?;
                wrap_for_recipients(&kms_ciphertext, recipients)
            }

            #[cfg(feature = "aws")]
            Self::HybridAws { key_id } => {
                use super::Cipher;
                let age_ct = Self::encrypt_age(plaintext, recipients)?;
                let kms = super::aws::AwsKms::new(key_id.clone());
                let kms_ct = kms.encrypt(plaintext, &[])?;
                Envelope::new(age_ct, Some(kms_ct), Some(&super::kms::KmsProvider::Aws)).seal()
            }

            #[cfg(feature = "gcp")]
            Self::GcpKms { resource } => {
                use super::Cipher;
                let gcp = super::gcp::GcpKms::new(resource.clone());
                let kms_ciphertext = gcp.encrypt(plaintext, &[])?;
                wrap_for_recipients(&kms_ciphertext, recipients)
            }

            #[cfg(feature = "gcp")]
            Self::HybridGcp { resource } => {
                use super::Cipher;
                let age_ct = Self::encrypt_age(plaintext, recipients)?;
                let gcp = super::gcp::GcpKms::new(resource.clone());
                let kms_ct = gcp.encrypt(plaintext, &[])?;
                Envelope::new(age_ct, Some(kms_ct), Some(&super::kms::KmsProvider::Gcp)).seal()
            }

            #[cfg(feature = "gpg")]
            Self::Gpg => {
                use super::Cipher;
                super::gpg::Gpg.encrypt(plaintext, recipients)
            }
        }
    }

    /// Decrypt ciphertext using the provided identity.
    ///
    /// For hybrid envelopes, tries the age path first (fast, local),
    /// then falls back to KMS if age decryption fails.
    /// For raw age ciphertext, decrypts directly.
    ///
    /// # Errors
    ///
    /// Returns `CipherError` if decryption fails.
    pub fn decrypt(&self, ciphertext: &str, identity: &age::x25519::Identity) -> Result<String> {
        use super::Cipher;
        #[allow(unused_imports)]
        use crate::core::cipher::kms::Envelope;

        match self {
            Self::Age => {
                // Check for envelope (backward compat: might have been encrypted in hybrid mode)
                if let Some(env) = Envelope::parse(ciphertext) {
                    return super::Age.decrypt(&env.age, identity);
                }
                super::Age.decrypt(ciphertext, identity)
            }

            #[cfg(any(test, feature = "test-kms"))]
            Self::Hybrid { kms, .. } => {
                if let Some(env) = Envelope::parse(ciphertext) {
                    // Try age first
                    if let Ok(result) = super::Age.decrypt(&env.age, identity) {
                        return Ok(result);
                    }
                    // Fall back to KMS
                    if let Some(kms_ct) = &env.kms {
                        use super::kms::KmsBackend;
                        return kms.decrypt(kms_ct);
                    }
                }
                // Raw age ciphertext
                super::Age.decrypt(ciphertext, identity)
            }

            #[cfg(feature = "aws")]
            Self::AwsKms { .. } => {
                let kms = super::aws::AwsKms::new(String::new());
                let kms_ciphertext = unwrap_for_identity(ciphertext, identity)?
                    .unwrap_or_else(|| ciphertext.to_string());
                kms.decrypt(&kms_ciphertext, &())
            }

            #[cfg(feature = "aws")]
            Self::HybridAws { key_id } => {
                if let Some(env) = Envelope::parse(ciphertext) {
                    if let Ok(result) = super::Age.decrypt(&env.age, identity) {
                        return Ok(result);
                    }
                    if let Some(kms_ct) = &env.kms {
                        let kms = super::aws::AwsKms::new(key_id.clone());
                        return kms.decrypt(kms_ct, &());
                    }
                }
                super::Age.decrypt(ciphertext, identity)
            }

            #[cfg(feature = "gcp")]
            Self::GcpKms { resource } => {
                let gcp = super::gcp::GcpKms::new(resource.clone());
                let kms_ciphertext = unwrap_for_identity(ciphertext, identity)?
                    .unwrap_or_else(|| ciphertext.to_string());
                gcp.decrypt(&kms_ciphertext, &())
            }

            #[cfg(feature = "gcp")]
            Self::HybridGcp { resource } => {
                if let Some(env) = Envelope::parse(ciphertext) {
                    if let Ok(result) = super::Age.decrypt(&env.age, identity) {
                        return Ok(result);
                    }
                    if let Some(kms_ct) = &env.kms {
                        let gcp = super::gcp::GcpKms::new(resource.clone());
                        return gcp.decrypt(kms_ct, &());
                    }
                }
                super::Age.decrypt(ciphertext, identity)
            }

            #[cfg(feature = "gpg")]
            Self::Gpg => super::gpg::Gpg.decrypt(ciphertext, &()),
        }
    }

    /// Backend name for display.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Age => "age",
            #[cfg(any(test, feature = "test-kms"))]
            Self::Hybrid { .. } => "hybrid",
            #[cfg(feature = "aws")]
            Self::AwsKms { .. } => "aws-kms",
            #[cfg(feature = "aws")]
            Self::HybridAws { .. } => "hybrid+aws",
            #[cfg(feature = "gcp")]
            Self::GcpKms { .. } => "gcp-kms",
            #[cfg(feature = "gcp")]
            Self::HybridGcp { .. } => "hybrid+gcp",
            #[cfg(feature = "gpg")]
            Self::Gpg => "gpg",
        }
    }
}

#[cfg(any(test, feature = "test-kms"))]
mod tests {
    use super::*;

    #[test]
    fn test_backend_from_config_default_age() {
        let config = Config::new();
        let backend = CipherBackend::from_config(&config).unwrap();
        assert_eq!(backend.name(), "age");
    }

    #[test]
    fn test_backend_from_config_explicit_age() {
        let mut config = Config::new();
        config.dugout.cipher = Some("age".to_string());
        let backend = CipherBackend::from_config(&config).unwrap();
        assert_eq!(backend.name(), "age");
    }

    #[test]
    #[cfg(feature = "aws")]
    fn test_backend_from_config_aws_kms() {
        let mut config = Config::new();
        config.dugout.cipher = Some("aws-kms".to_string());
        config.dugout.kms_key_id = Some("arn:aws:kms:us-east-1:123456789012:key/test".to_string());
        let backend = CipherBackend::from_config(&config).unwrap();
        assert_eq!(backend.name(), "aws-kms");
    }

    #[test]
    #[cfg(feature = "aws")]
    fn test_backend_from_config_aws_kms_missing_key() {
        let mut config = Config::new();
        config.dugout.cipher = Some("aws-kms".to_string());
        let result = CipherBackend::from_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_backend_from_config_unknown_cipher() {
        let mut config = Config::new();
        config.dugout.cipher = Some("unknown".to_string());
        let result = CipherBackend::from_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_backend_encrypt_decrypt_age() {
        let config = Config::new();
        let backend = CipherBackend::from_config(&config).unwrap();

        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public().to_string();

        let plaintext = "test secret";
        let encrypted = backend.encrypt(plaintext, &[recipient]).unwrap();
        let decrypted = backend.decrypt(&encrypted, &identity).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrap_roundtrip_for_identity() {
        let identity = age::x25519::Identity::generate();
        let recipients = vec![identity.to_public().to_string()];
        let kms_ciphertext = "kms-base64-ciphertext";

        let wrapped = wrap_for_recipients(kms_ciphertext, &recipients).unwrap();
        let unwrapped = unwrap_for_identity(&wrapped, &identity)
            .unwrap()
            .expect("wrapped payload should unwrap");

        assert_eq!(unwrapped, kms_ciphertext);
    }

    #[test]
    fn test_wrap_requires_recipients() {
        let kms_ciphertext = "kms-base64-ciphertext";
        let recipients: Vec<String> = Vec::new();

        let result = wrap_for_recipients(kms_ciphertext, &recipients);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrap_rejects_invalid_recipient_key() {
        let kms_ciphertext = "kms-base64-ciphertext";
        let recipients = vec!["not-an-age-key".to_string()];

        let result = wrap_for_recipients(kms_ciphertext, &recipients);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrap_rejects_wrong_identity() {
        let identity = age::x25519::Identity::generate();
        let outsider = age::x25519::Identity::generate();
        let recipients = vec![identity.to_public().to_string()];
        let kms_ciphertext = "kms-base64-ciphertext";

        let wrapped = wrap_for_recipients(kms_ciphertext, &recipients).unwrap();
        let result = unwrap_for_identity(&wrapped, &outsider);

        assert!(result.is_err());
    }

    #[test]
    fn test_unwrap_legacy_ciphertext_passthrough() {
        let identity = age::x25519::Identity::generate();
        let legacy = "legacy-kms-ciphertext";

        let unwrapped = unwrap_for_identity(legacy, &identity).unwrap();
        assert!(unwrapped.is_none());
    }

    #[test]
    fn test_unwrap_rejects_unknown_wrapper_version() {
        let identity = age::x25519::Identity::generate();
        let payload = r#"{"version":"dugout-ciphertext-envelope-v0","ciphertext":"x"}"#;

        let result = unwrap_for_identity(payload, &identity);
        assert!(result.is_err());
    }

    #[test]
    fn test_unwrap_rejects_legacy_wrapper_version() {
        let identity = age::x25519::Identity::generate();
        let recipients = vec![identity.to_public().to_string()];
        let wrapped = wrap_for_recipients("kms-base64-ciphertext", &recipients).unwrap();
        let wrapped: WrappedCiphertext = serde_json::from_str(&wrapped).unwrap();
        let payload = serde_json::json!({
            "version": "dugout-kms-wrap-v1",
            "ciphertext": wrapped.ciphertext,
        })
        .to_string();

        let result = unwrap_for_identity(&payload, &identity);
        assert!(result.is_err());
    }

    #[test]
    fn test_unwrap_rejects_legacy_field_name() {
        let identity = age::x25519::Identity::generate();
        let payload = r#"{"version":"dugout-ciphertext-envelope-v1","wrapped_kms_ciphertext":"x"}"#;

        let result = unwrap_for_identity(payload, &identity);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(not(feature = "aws"))]
    fn test_backend_aws_kms_not_compiled() {
        let mut config = Config::new();
        config.dugout.cipher = Some("aws-kms".to_string());
        config.dugout.kms_key_id = Some("test-key".to_string());

        let result = CipherBackend::from_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not compiled"));
    }

    #[test]
    #[cfg(not(feature = "gcp"))]
    fn test_backend_gcp_kms_not_compiled() {
        let mut config = Config::new();
        config.dugout.cipher = Some("gcp-kms".to_string());
        config.dugout.gcp_resource = Some("test-resource".to_string());

        let result = CipherBackend::from_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not compiled"));
    }

    #[test]
    #[cfg(not(feature = "gpg"))]
    fn test_backend_gpg_not_compiled() {
        let mut config = Config::new();
        config.dugout.cipher = Some("gpg".to_string());

        let result = CipherBackend::from_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not compiled"));
    }

    // --- Hybrid mode tests ---

    #[test]
    fn test_hybrid_from_config() {
        use crate::core::config::KmsConfig;

        let mut config = Config::new();
        config.kms = Some(KmsConfig {
            key: "arn:aws:kms:us-east-1:123:key/abc".to_string(),
        });
        // Need at least one recipient for config validation
        let identity = age::x25519::Identity::generate();
        config
            .recipients
            .insert("test".to_string(), identity.to_public().to_string());

        let backend = CipherBackend::from_config(&config).unwrap();
        assert_eq!(backend.name(), "hybrid");
    }

    #[test]
    fn test_hybrid_encrypt_produces_envelope() {
        use crate::core::cipher::kms::Envelope;
        use crate::core::config::KmsConfig;

        let mut config = Config::new();
        config.kms = Some(KmsConfig {
            key: "arn:aws:kms:us-east-1:123:key/abc".to_string(),
        });
        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public().to_string();
        config
            .recipients
            .insert("test".to_string(), recipient.clone());

        let backend = CipherBackend::from_config(&config).unwrap();
        let encrypted = backend.encrypt("my-secret", &[recipient]).unwrap();

        // Should be an envelope
        let envelope = Envelope::parse(&encrypted).expect("should be an envelope");
        assert!(envelope.kms.is_some());
        assert_eq!(envelope.provider.as_deref(), Some("aws"));
    }

    #[test]
    fn test_hybrid_decrypt_via_age() {
        use crate::core::config::KmsConfig;

        let mut config = Config::new();
        config.kms = Some(KmsConfig {
            key: "arn:aws:kms:us-east-1:123:key/abc".to_string(),
        });
        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public().to_string();
        config
            .recipients
            .insert("test".to_string(), recipient.clone());

        let backend = CipherBackend::from_config(&config).unwrap();
        let encrypted = backend.encrypt("hybrid-secret", &[recipient]).unwrap();
        let decrypted = backend.decrypt(&encrypted, &identity).unwrap();

        assert_eq!(decrypted, "hybrid-secret");
    }

    #[test]
    fn test_hybrid_decrypt_via_kms_fallback() {
        use crate::core::config::KmsConfig;

        let mut config = Config::new();
        config.kms = Some(KmsConfig {
            key: "arn:aws:kms:us-east-1:123:key/abc".to_string(),
        });
        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public().to_string();
        config
            .recipients
            .insert("test".to_string(), recipient.clone());

        let backend = CipherBackend::from_config(&config).unwrap();
        let encrypted = backend
            .encrypt("kms-fallback-secret", &[recipient])
            .unwrap();

        // Use a DIFFERENT identity (simulating a server without the right age key)
        let wrong_identity = age::x25519::Identity::generate();
        let decrypted = backend.decrypt(&encrypted, &wrong_identity).unwrap();

        assert_eq!(decrypted, "kms-fallback-secret");
    }

    #[test]
    fn test_hybrid_backward_compat_raw_age() {
        use crate::core::config::KmsConfig;

        // Create a secret with age-only backend
        let age_config = Config::new();
        let age_backend = CipherBackend::from_config(&age_config).unwrap();
        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public().to_string();
        let age_encrypted = age_backend
            .encrypt("old-secret", &[recipient.clone()])
            .unwrap();

        // Now open with hybrid backend — should still decrypt raw age
        let mut hybrid_config = Config::new();
        hybrid_config.kms = Some(KmsConfig {
            key: "arn:aws:kms:us-east-1:123:key/abc".to_string(),
        });
        hybrid_config
            .recipients
            .insert("test".to_string(), recipient);
        let hybrid_backend = CipherBackend::from_config(&hybrid_config).unwrap();

        let decrypted = hybrid_backend.decrypt(&age_encrypted, &identity).unwrap();
        assert_eq!(decrypted, "old-secret");
    }

    #[test]
    fn test_age_backend_reads_hybrid_envelope() {
        use crate::core::config::KmsConfig;

        // Encrypt with hybrid
        let mut config = Config::new();
        config.kms = Some(KmsConfig {
            key: "arn:aws:kms:us-east-1:123:key/abc".to_string(),
        });
        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public().to_string();
        config
            .recipients
            .insert("test".to_string(), recipient.clone());
        let hybrid_backend = CipherBackend::from_config(&config).unwrap();
        let encrypted = hybrid_backend
            .encrypt("cross-compat", &[recipient])
            .unwrap();

        // Decrypt with age-only backend (simulating dev who doesn't have KMS feature)
        let age_backend = CipherBackend::Age;
        let decrypted = age_backend.decrypt(&encrypted, &identity).unwrap();

        assert_eq!(decrypted, "cross-compat");
    }

    #[test]
    fn test_hybrid_invalid_kms_key_format() {
        use crate::core::config::KmsConfig;

        let mut config = Config::new();
        config.kms = Some(KmsConfig {
            key: "not-a-valid-kms-key".to_string(),
        });

        let result = CipherBackend::from_config(&config);
        assert!(result.is_err());
    }
}
