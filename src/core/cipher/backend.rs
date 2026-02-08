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

/// Vault cipher selection and dispatch
///
/// Wraps the different cipher implementations and provides
/// dynamic dispatch based on configuration.
#[derive(Debug)]
pub enum CipherBackend {
    /// Age encryption (default, always available)
    Age,

    #[cfg(feature = "aws")]
    /// AWS KMS encryption
    AwsKms { key_id: String },

    #[cfg(feature = "gcp")]
    /// GCP KMS encryption
    GcpKms { resource: String },

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

        debug!(cipher = %cipher_type, "creating cipher backend");

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

    /// Encrypt plaintext for the given recipients
    ///
    /// # Errors
    ///
    /// Returns `CipherError` if encryption fails.
    pub fn encrypt(&self, plaintext: &str, recipients: &[String]) -> Result<String> {
        use super::Cipher;

        match self {
            Self::Age => {
                // Parse age recipients
                let age_recipients: Result<Vec<_>> = recipients
                    .iter()
                    .map(|r| super::parse_recipient(r))
                    .collect();
                let age_recipients = age_recipients?;
                super::Age.encrypt(plaintext, &age_recipients)
            }

            #[cfg(feature = "aws")]
            Self::AwsKms { key_id } => {
                let kms = super::aws::AwsKms::new(key_id.clone());
                let kms_ciphertext = kms.encrypt(plaintext, &[])?;
                wrap_for_recipients(&kms_ciphertext, recipients)
            }

            #[cfg(feature = "gcp")]
            Self::GcpKms { resource } => {
                let gcp = super::gcp::GcpKms::new(resource.clone());
                let kms_ciphertext = gcp.encrypt(plaintext, &[])?;
                wrap_for_recipients(&kms_ciphertext, recipients)
            }

            #[cfg(feature = "gpg")]
            Self::Gpg => super::gpg::Gpg.encrypt(plaintext, recipients),
        }
    }

    /// Decrypt ciphertext using the provided identity
    ///
    /// # Errors
    ///
    /// Returns `CipherError` if decryption fails.
    pub fn decrypt(&self, ciphertext: &str, identity: &age::x25519::Identity) -> Result<String> {
        use super::Cipher;

        match self {
            Self::Age => super::Age.decrypt(ciphertext, identity),

            #[cfg(feature = "aws")]
            Self::AwsKms { .. } => {
                let kms = super::aws::AwsKms::new(String::new()); // key_id not needed for decrypt
                let kms_ciphertext = unwrap_for_identity(ciphertext, identity)?
                    .unwrap_or_else(|| ciphertext.to_string());
                kms.decrypt(&kms_ciphertext, &())
            }

            #[cfg(feature = "gcp")]
            Self::GcpKms { resource } => {
                let gcp = super::gcp::GcpKms::new(resource.clone());
                let kms_ciphertext = unwrap_for_identity(ciphertext, identity)?
                    .unwrap_or_else(|| ciphertext.to_string());
                gcp.decrypt(&kms_ciphertext, &())
            }

            #[cfg(feature = "gpg")]
            Self::Gpg => super::gpg::Gpg.decrypt(ciphertext, &()),
        }
    }

    /// Backend name for display
    pub fn name(&self) -> &'static str {
        match self {
            Self::Age => "age",
            #[cfg(feature = "aws")]
            Self::AwsKms { .. } => "aws-kms",
            #[cfg(feature = "gcp")]
            Self::GcpKms { .. } => "gcp-kms",
            #[cfg(feature = "gpg")]
            Self::Gpg => "gpg",
        }
    }
}

#[cfg(test)]
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
}
