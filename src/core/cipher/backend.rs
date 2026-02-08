//! Cipher backend selection and dispatch.
//!
//! Two modes:
//! - **Age** (default): secrets encrypted with age only
//! - **Hybrid**: secrets encrypted with age + cloud KMS

use crate::core::config::Config;
use crate::error::{CipherError, Result};
use tracing::debug;

use super::kms::envelope::{Envelope, KmsProvider};

/// Cipher backend for vault operations.
///
/// - `Age`: raw age ciphertext (default)
/// - `Hybrid`: v2 envelope with age + cloud KMS
/// - `Gpg`: GPG encryption via gpg CLI
#[derive(Debug)]
pub enum CipherBackend {
    /// Age encryption (default)
    Age,

    /// Hybrid: age + cloud KMS
    #[allow(dead_code)]
    Hybrid { provider: KmsProvider, key: String },

    /// GPG encryption
    #[cfg(feature = "gpg")]
    Gpg,
}

impl CipherBackend {
    /// Create a cipher backend from configuration.
    pub fn from_config(config: &Config) -> Result<Self> {
        // Check for explicit cipher override
        if let Some(cipher) = config.cipher() {
            match cipher {
                "gpg" => {
                    #[cfg(feature = "gpg")]
                    {
                        debug!("creating gpg cipher backend");
                        return Ok(Self::Gpg);
                    }
                    #[cfg(not(feature = "gpg"))]
                    {
                        return Err(CipherError::EncryptionFailed(
                            "GPG support not compiled. Rebuild with: cargo install dugout --features gpg".to_string()
                        ).into());
                    }
                }
                "age" => {} // fall through to age/hybrid logic
                other => {
                    return Err(CipherError::EncryptionFailed(format!(
                        "unknown cipher: {}. Supported: age, gpg",
                        other
                    ))
                    .into());
                }
            }
        }

        // Check for hybrid mode (age + KMS)
        if let Some(kms_key) = config.kms_key() {
            debug!(kms_key = %kms_key, "creating hybrid cipher backend");
            let provider = KmsProvider::detect(kms_key).ok_or_else(|| {
                CipherError::EncryptionFailed(format!(
                    "unrecognized KMS key format: {}. Expected AWS ARN (arn:aws:kms:...) or GCP resource (projects/...)",
                    kms_key
                ))
            })?;
            return Ok(Self::Hybrid {
                provider,
                key: kms_key.to_string(),
            });
        }

        debug!("creating age cipher backend");
        Ok(Self::Age)
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

    /// Encrypt plaintext using the configured KMS backend.
    #[allow(unused_variables)]
    fn encrypt_kms(&self, plaintext: &str) -> Result<String> {
        match self {
            Self::Age => unreachable!("encrypt_kms called on Age backend"),

            #[cfg(any(test, feature = "test-kms"))]
            Self::Hybrid { .. } => {
                use super::kms::envelope::{KmsBackend, MockKms};
                MockKms.encrypt(plaintext)
            }

            #[cfg(all(not(test), not(feature = "test-kms"), feature = "aws"))]
            Self::Hybrid {
                provider: KmsProvider::Aws,
                key,
            } => {
                use super::Cipher;
                super::kms::aws::AwsKms::new(key.clone()).encrypt(plaintext, &[])
            }

            #[cfg(all(not(test), not(feature = "test-kms"), feature = "gcp"))]
            Self::Hybrid {
                provider: KmsProvider::Gcp,
                key,
            } => {
                use super::Cipher;
                super::kms::gcp::GcpKms::new(key.clone()).encrypt(plaintext, &[])
            }

            #[cfg(all(not(test), not(feature = "test-kms")))]
            Self::Hybrid { provider, .. } => Err(CipherError::EncryptionFailed(format!(
                "{} KMS not compiled. Rebuild with: cargo install dugout --features {}",
                provider.name(),
                provider.name()
            ))
            .into()),
        }
    }

    /// Decrypt ciphertext using the configured KMS backend.
    #[allow(unused_variables)]
    fn decrypt_kms(&self, ciphertext: &str) -> Result<String> {
        match self {
            Self::Age => unreachable!("decrypt_kms called on Age backend"),

            #[cfg(any(test, feature = "test-kms"))]
            Self::Hybrid { .. } => {
                use super::kms::envelope::{KmsBackend, MockKms};
                MockKms.decrypt(ciphertext)
            }

            #[cfg(all(not(test), not(feature = "test-kms"), feature = "aws"))]
            Self::Hybrid {
                provider: KmsProvider::Aws,
                ..
            } => {
                use super::Cipher;
                super::kms::aws::AwsKms::new(String::new()).decrypt(ciphertext, &())
            }

            #[cfg(all(not(test), not(feature = "test-kms"), feature = "gcp"))]
            Self::Hybrid {
                provider: KmsProvider::Gcp,
                key,
            } => {
                use super::Cipher;
                super::kms::gcp::GcpKms::new(key.clone()).decrypt(ciphertext, &())
            }

            #[cfg(all(not(test), not(feature = "test-kms")))]
            Self::Hybrid { provider, .. } => Err(CipherError::DecryptionFailed(format!(
                "{} KMS not compiled. Rebuild with: cargo install dugout --features {}",
                provider.name(),
                provider.name()
            ))
            .into()),
        }
    }

    /// Encrypt plaintext for the given recipients.
    ///
    /// - Age mode: raw age ciphertext
    /// - Hybrid mode: v2 envelope with age + KMS ciphertext
    pub fn encrypt(&self, plaintext: &str, recipients: &[String]) -> Result<String> {
        match self {
            Self::Age => Self::encrypt_age(plaintext, recipients),

            #[cfg(feature = "gpg")]
            Self::Gpg => {
                use super::Cipher;
                super::gpg::Gpg.encrypt(plaintext, recipients)
            }

            Self::Hybrid { provider, .. } => {
                let age_ct = Self::encrypt_age(plaintext, recipients)?;
                let kms_ct = self.encrypt_kms(plaintext)?;
                Envelope::new(age_ct, Some(kms_ct), Some(provider)).seal()
            }
        }
    }

    /// Decrypt ciphertext using the provided identity.
    ///
    /// For envelopes, tries age first (fast, local), then KMS.
    /// For raw age ciphertext, decrypts directly.
    pub fn decrypt(&self, ciphertext: &str, identity: &age::x25519::Identity) -> Result<String> {
        use super::Cipher;

        #[cfg(feature = "gpg")]
        if let Self::Gpg = self {
            use super::Cipher;
            return super::gpg::Gpg.decrypt(ciphertext, &());
        }

        if let Some(env) = Envelope::parse(ciphertext) {
            // Envelope: try age, then KMS
            if let Ok(result) = super::Age.decrypt(&env.age, identity) {
                return Ok(result);
            }
            if let Some(kms_ct) = &env.kms {
                return self.decrypt_kms(kms_ct);
            }
            return Err(CipherError::DecryptionFailed(
                "envelope decryption failed: no valid path".to_string(),
            )
            .into());
        }

        // Raw age ciphertext
        super::Age.decrypt(ciphertext, identity)
    }

    /// Backend name for display.
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Age => "age",
            Self::Hybrid { provider, .. } => match provider {
                KmsProvider::Aws => "hybrid+aws",
                KmsProvider::Gcp => "hybrid+gcp",
            },
            #[cfg(feature = "gpg")]
            Self::Gpg => "gpg",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::KmsConfig;

    #[test]
    fn test_age_from_config() {
        let config = Config::new();
        let backend = CipherBackend::from_config(&config).unwrap();
        assert_eq!(backend.name(), "age");
    }

    #[test]
    fn test_hybrid_from_config() {
        let mut config = Config::new();
        config.kms = Some(KmsConfig {
            key: "arn:aws:kms:us-east-1:123:key/abc".to_string(),
        });
        let backend = CipherBackend::from_config(&config).unwrap();
        assert_eq!(backend.name(), "hybrid+aws");
    }

    #[test]
    fn test_hybrid_gcp_from_config() {
        let mut config = Config::new();
        config.kms = Some(KmsConfig {
            key: "projects/my-proj/locations/global/keyRings/ring/cryptoKeys/key".to_string(),
        });
        let backend = CipherBackend::from_config(&config).unwrap();
        assert_eq!(backend.name(), "hybrid+gcp");
    }

    #[test]
    fn test_invalid_kms_key_format() {
        let mut config = Config::new();
        config.kms = Some(KmsConfig {
            key: "not-a-valid-kms-key".to_string(),
        });
        assert!(CipherBackend::from_config(&config).is_err());
    }

    #[test]
    fn test_age_encrypt_decrypt() {
        let backend = CipherBackend::Age;
        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public().to_string();

        let encrypted = backend.encrypt("test secret", &[recipient]).unwrap();
        let decrypted = backend.decrypt(&encrypted, &identity).unwrap();
        assert_eq!(decrypted, "test secret");
    }

    #[test]
    fn test_hybrid_encrypt_produces_envelope() {
        let mut config = Config::new();
        config.kms = Some(KmsConfig {
            key: "arn:aws:kms:us-east-1:123:key/abc".to_string(),
        });
        let backend = CipherBackend::from_config(&config).unwrap();
        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public().to_string();

        let encrypted = backend.encrypt("my-secret", &[recipient]).unwrap();
        let envelope = Envelope::parse(&encrypted).expect("should be envelope");
        assert!(envelope.kms.is_some());
        assert_eq!(envelope.provider.as_deref(), Some("aws"));
    }

    #[test]
    fn test_hybrid_decrypt_via_age() {
        let mut config = Config::new();
        config.kms = Some(KmsConfig {
            key: "arn:aws:kms:us-east-1:123:key/abc".to_string(),
        });
        let backend = CipherBackend::from_config(&config).unwrap();
        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public().to_string();

        let encrypted = backend.encrypt("hybrid-secret", &[recipient]).unwrap();
        let decrypted = backend.decrypt(&encrypted, &identity).unwrap();
        assert_eq!(decrypted, "hybrid-secret");
    }

    #[test]
    fn test_hybrid_decrypt_via_kms_fallback() {
        let mut config = Config::new();
        config.kms = Some(KmsConfig {
            key: "arn:aws:kms:us-east-1:123:key/abc".to_string(),
        });
        let backend = CipherBackend::from_config(&config).unwrap();
        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public().to_string();

        let encrypted = backend.encrypt("kms-fallback", &[recipient]).unwrap();
        let wrong_identity = age::x25519::Identity::generate();
        let decrypted = backend.decrypt(&encrypted, &wrong_identity).unwrap();
        assert_eq!(decrypted, "kms-fallback");
    }

    #[test]
    fn test_hybrid_backward_compat_raw_age() {
        // Encrypt with age-only
        let age_backend = CipherBackend::Age;
        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public().to_string();
        let age_encrypted = age_backend.encrypt("old-secret", &[recipient]).unwrap();

        // Decrypt with hybrid backend
        let mut config = Config::new();
        config.kms = Some(KmsConfig {
            key: "arn:aws:kms:us-east-1:123:key/abc".to_string(),
        });
        let hybrid_backend = CipherBackend::from_config(&config).unwrap();
        let decrypted = hybrid_backend.decrypt(&age_encrypted, &identity).unwrap();
        assert_eq!(decrypted, "old-secret");
    }

    #[test]
    fn test_age_reads_hybrid_envelope() {
        // Encrypt with hybrid
        let mut config = Config::new();
        config.kms = Some(KmsConfig {
            key: "arn:aws:kms:us-east-1:123:key/abc".to_string(),
        });
        let hybrid_backend = CipherBackend::from_config(&config).unwrap();
        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public().to_string();
        let encrypted = hybrid_backend
            .encrypt("cross-compat", &[recipient])
            .unwrap();

        // Decrypt with age-only
        let age_backend = CipherBackend::Age;
        let decrypted = age_backend.decrypt(&encrypted, &identity).unwrap();
        assert_eq!(decrypted, "cross-compat");
    }
}
