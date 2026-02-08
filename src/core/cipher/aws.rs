//! AWS KMS cipher backend.
//!
//! Encrypts secrets using AWS Key Management Service.
//! Enable with `--features aws`.
//!
//! ## Usage
//!
//! Configure your vault with:
//! ```toml
//! [meta]
//! cipher = "aws-kms"
//!
//! [meta.kms]
//! key_id = "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012"
//! ```
//!
//! The AWS KMS cipher uses AWS credentials from the environment (AWS_ACCESS_KEY_ID, etc.)
//! or from the default credential provider chain.

use tracing::trace;

use super::Cipher;
use crate::error::{CipherError, Result};

/// AWS KMS cipher backend
///
/// Uses AWS Key Management Service for encryption/decryption.
/// KMS stores the key information in the ciphertext, so decryption
/// doesn't require specifying the key ID.
#[cfg(feature = "aws")]
#[allow(dead_code)]
pub struct AwsKms {
    key_id: String,
}

#[cfg(feature = "aws")]
impl AwsKms {
    /// Create a new AWS KMS cipher with the specified key ID or ARN
    #[allow(dead_code)]
    pub fn new(key_id: String) -> Self {
        Self { key_id }
    }
}

#[cfg(feature = "aws")]
impl Cipher for AwsKms {
    // KMS doesn't use traditional recipients - the key_id is used for encryption
    // For compatibility with the trait, we use String for both
    type Recipient = String;
    type Identity = ();

    fn name(&self) -> &'static str {
        "aws-kms"
    }

    fn encrypt(&self, plaintext: &str, _recipients: &[String]) -> Result<String> {
        use ::base64::Engine;

        trace!(
            key_id = %self.key_id,
            plaintext_len = plaintext.len(),
            "encrypting with AWS KMS"
        );

        // Create a tokio runtime for the async AWS SDK
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                CipherError::EncryptionFailed(format!("failed to create runtime: {}", e))
            })?;

        rt.block_on(async {
            // Load AWS config from environment
            let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
            let client = aws_sdk_kms::Client::new(&config);

            // Encrypt the plaintext
            let result = client
                .encrypt()
                .key_id(&self.key_id)
                .plaintext(aws_sdk_kms::primitives::Blob::new(plaintext.as_bytes()))
                .send()
                .await
                .map_err(|e| CipherError::EncryptionFailed(format!("KMS encrypt failed: {}", e)))?;

            // Extract the ciphertext blob
            let blob = result
                .ciphertext_blob()
                .ok_or_else(|| CipherError::EncryptionFailed("no ciphertext returned".into()))?;

            // Encode as base64 for storage
            let encoded = ::base64::engine::general_purpose::STANDARD.encode(blob.as_ref());

            trace!(ciphertext_len = encoded.len(), "encrypted with AWS KMS");
            Ok(encoded)
        })
    }

    fn decrypt(&self, ciphertext: &str, _identity: &()) -> Result<String> {
        trace!(ciphertext_len = ciphertext.len(), "decrypting with AWS KMS");

        // Decode from base64
        use ::base64::Engine;
        let blob = ::base64::engine::general_purpose::STANDARD
            .decode(ciphertext)
            .map_err(|e| CipherError::DecryptionFailed(format!("invalid base64: {}", e)))?;

        // Create a tokio runtime for the async AWS SDK
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                CipherError::DecryptionFailed(format!("failed to create runtime: {}", e))
            })?;

        rt.block_on(async {
            // Load AWS config from environment
            let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
            let client = aws_sdk_kms::Client::new(&config);

            // Decrypt the ciphertext
            // KMS stores the key ID in the ciphertext blob, so we don't need to specify it
            let result = client
                .decrypt()
                .ciphertext_blob(aws_sdk_kms::primitives::Blob::new(blob))
                .send()
                .await
                .map_err(|e| CipherError::DecryptionFailed(format!("KMS decrypt failed: {}", e)))?;

            // Extract the plaintext
            let plaintext_blob = result
                .plaintext()
                .ok_or_else(|| CipherError::DecryptionFailed("no plaintext returned".into()))?;

            // Convert to string
            let plaintext = String::from_utf8(plaintext_blob.as_ref().to_vec())
                .map_err(|e| CipherError::DecryptionFailed(format!("UTF-8 error: {}", e)))?;

            trace!(plaintext_len = plaintext.len(), "decrypted with AWS KMS");
            Ok(plaintext)
        })
    }
}
