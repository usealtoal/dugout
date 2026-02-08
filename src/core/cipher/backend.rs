//! Cipher backend selection and dispatch.
//!
//! Provides a unified interface for multiple cipher backends.

use crate::core::config::Config;
#[cfg(any(feature = "aws", feature = "gcp"))]
use crate::error::ConfigError;
use crate::error::{CipherError, Result};
use tracing::debug;

/// Cipher backend selection.
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
    /// Create a cipher backend from configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The burrow configuration
    ///
    /// # Returns
    ///
    /// The appropriate cipher backend based on config.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Unknown cipher type specified
    /// - Required configuration fields are missing
    /// - Cipher type requires a feature that wasn't compiled in
    pub fn from_config(config: &Config) -> Result<Self> {
        let cipher_type = config.burrow.cipher.as_deref().unwrap_or("age");

        debug!(cipher = %cipher_type, "creating cipher backend");

        match cipher_type {
            "age" => Ok(Self::Age),

            "aws-kms" => {
                #[cfg(feature = "aws")]
                {
                    let key_id = config.burrow.kms_key_id.clone().ok_or_else(|| {
                        ConfigError::MissingField {
                            field: "kms_key_id",
                        }
                    })?;
                    Ok(Self::AwsKms { key_id })
                }
                #[cfg(not(feature = "aws"))]
                {
                    Err(CipherError::EncryptionFailed(
                        "AWS KMS support not compiled. Rebuild with: cargo install burrow --features aws".to_string()
                    ).into())
                }
            }

            "gcp-kms" => {
                #[cfg(feature = "gcp")]
                {
                    let resource = config.burrow.gcp_resource.clone().ok_or_else(|| {
                        ConfigError::MissingField {
                            field: "gcp_resource",
                        }
                    })?;
                    Ok(Self::GcpKms { resource })
                }
                #[cfg(not(feature = "gcp"))]
                {
                    Err(CipherError::EncryptionFailed(
                        "GCP KMS support not compiled. Rebuild with: cargo install burrow --features gcp".to_string()
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
                        "GPG support not compiled. Rebuild with: cargo install burrow --features gpg".to_string()
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

    /// Encrypt plaintext.
    ///
    /// # Arguments
    ///
    /// * `plaintext` - The string to encrypt
    /// * `recipients` - List of recipient identifiers (format depends on backend)
    ///
    /// # Returns
    ///
    /// Encrypted string.
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
                // KMS doesn't use recipients in the traditional sense
                // Just pass empty vector for the trait interface
                kms.encrypt(plaintext, &[])
            }

            #[cfg(feature = "gcp")]
            Self::GcpKms { resource } => {
                let gcp = super::gcp::GcpKms::new(resource.clone());
                gcp.encrypt(plaintext, &[])
            }

            #[cfg(feature = "gpg")]
            Self::Gpg => super::gpg::Gpg.encrypt(plaintext, recipients),
        }
    }

    /// Decrypt ciphertext.
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - The encrypted string
    /// * `identity` - The age identity (used only for age backend)
    ///
    /// # Returns
    ///
    /// Decrypted plaintext.
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
                kms.decrypt(ciphertext, &())
            }

            #[cfg(feature = "gcp")]
            Self::GcpKms { resource } => {
                let gcp = super::gcp::GcpKms::new(resource.clone());
                gcp.decrypt(ciphertext, &())
            }

            #[cfg(feature = "gpg")]
            Self::Gpg => super::gpg::Gpg.decrypt(ciphertext, &()),
        }
    }

    /// Get the backend name for display.
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
        config.burrow.cipher = Some("age".to_string());
        let backend = CipherBackend::from_config(&config).unwrap();
        assert_eq!(backend.name(), "age");
    }

    #[test]
    #[cfg(feature = "aws")]
    fn test_backend_from_config_aws_kms() {
        let mut config = Config::new();
        config.burrow.cipher = Some("aws-kms".to_string());
        config.burrow.kms_key_id = Some("arn:aws:kms:us-east-1:123456789012:key/test".to_string());
        let backend = CipherBackend::from_config(&config).unwrap();
        assert_eq!(backend.name(), "aws-kms");
    }

    #[test]
    #[cfg(feature = "aws")]
    fn test_backend_from_config_aws_kms_missing_key() {
        let mut config = Config::new();
        config.burrow.cipher = Some("aws-kms".to_string());
        let result = CipherBackend::from_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_backend_from_config_unknown_cipher() {
        let mut config = Config::new();
        config.burrow.cipher = Some("unknown".to_string());
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
    #[cfg(not(feature = "aws"))]
    fn test_backend_aws_kms_not_compiled() {
        let mut config = Config::new();
        config.burrow.cipher = Some("aws-kms".to_string());
        config.burrow.kms_key_id = Some("test-key".to_string());
        
        let result = CipherBackend::from_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not compiled"));
    }

    #[test]
    #[cfg(not(feature = "gcp"))]
    fn test_backend_gcp_kms_not_compiled() {
        let mut config = Config::new();
        config.burrow.cipher = Some("gcp-kms".to_string());
        config.burrow.gcp_resource = Some("test-resource".to_string());
        
        let result = CipherBackend::from_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not compiled"));
    }

    #[test]
    #[cfg(not(feature = "gpg"))]
    fn test_backend_gpg_not_compiled() {
        let mut config = Config::new();
        config.burrow.cipher = Some("gpg".to_string());
        
        let result = CipherBackend::from_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not compiled"));
    }
}
