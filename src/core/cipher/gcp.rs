//! Google Cloud KMS cipher backend.
//!
//! Encrypts secrets using Google Cloud Key Management Service via the gcloud CLI in hybrid mode.
//! Enable with `--features gcp`.
//!
//! ## Requirements
//!
//! - `gcloud` CLI must be installed and authenticated
//! - User must have cloudkms.cryptoKeyVersions.useToEncrypt and useToDecrypt permissions
//!
//! ## Usage
//!
//! Initialize with KMS key:
//! ```bash
//! dugout init --kms projects/my-project/locations/global/keyRings/my-ring/cryptoKeys/my-key
//! ```
//!
//! This creates a vault configuration with:
//! ```toml
//! [kms]
//! key = "projects/my-project/locations/global/keyRings/my-ring/cryptoKeys/my-key"
//! ```

use std::io::Write;
use std::process::{Command, Stdio};
use tracing::trace;

use crate::core::cipher::Cipher;
use crate::error::{CipherError, Result};

/// Google Cloud KMS cipher backend using gcloud CLI
#[cfg(feature = "gcp")]
#[allow(dead_code)]
pub struct GcpKms {
    /// Full resource name: projects/*/locations/*/keyRings/*/cryptoKeys/*
    resource_name: String,
}

#[cfg(feature = "gcp")]
impl GcpKms {
    /// Create a new GCP KMS cipher with the specified resource name
    ///
    /// # Example
    ///
    /// ```ignore
    /// let cipher = GcpKms::new(
    ///     "projects/my-project/locations/global/keyRings/my-ring/cryptoKeys/my-key".to_string()
    /// );
    /// ```
    #[allow(dead_code)]
    pub fn new(resource_name: String) -> Self {
        Self { resource_name }
    }

    /// Parse resource name into components for gcloud command
    #[allow(dead_code)]
    fn parse_resource_name(&self) -> Result<(String, String, String, String)> {
        let parts: Vec<&str> = self.resource_name.split('/').collect();

        if parts.len() != 8
            || parts[0] != "projects"
            || parts[2] != "locations"
            || parts[4] != "keyRings"
            || parts[6] != "cryptoKeys"
        {
            return Err(CipherError::EncryptionFailed(format!(
                "invalid GCP KMS resource name format: {}",
                self.resource_name
            ))
            .into());
        }

        Ok((
            parts[1].to_string(), // project
            parts[3].to_string(), // location
            parts[5].to_string(), // keyring
            parts[7].to_string(), // key
        ))
    }

    /// Check if gcloud CLI is available
    #[allow(dead_code)]
    fn check_gcloud() -> Result<()> {
        Command::new("gcloud")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|_| CipherError::EncryptionFailed(
                "gcloud CLI not found. Install it from https://cloud.google.com/sdk/docs/install".to_string()
            ))?;
        Ok(())
    }
}

#[cfg(feature = "gcp")]
impl Cipher for GcpKms {
    // GCP KMS uses the resource name for encryption, not traditional recipients
    type Recipient = String;
    type Identity = ();

    fn name(&self) -> &'static str {
        "gcp-kms"
    }

    fn encrypt(&self, plaintext: &str, _recipients: &[String]) -> Result<String> {
        use ::base64::Engine;

        trace!(
            resource_name = %self.resource_name,
            plaintext_len = plaintext.len(),
            "encrypting with GCP KMS"
        );

        Self::check_gcloud()?;

        let (project, location, keyring, key) = self.parse_resource_name()?;

        // Use gcloud to encrypt
        let mut child = Command::new("gcloud")
            .args([
                "kms",
                "encrypt",
                "--project",
                &project,
                "--location",
                &location,
                "--keyring",
                &keyring,
                "--key",
                &key,
                "--plaintext-file",
                "-",
                "--ciphertext-file",
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| CipherError::EncryptionFailed(format!("failed to spawn gcloud: {}", e)))?;

        // Write plaintext to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(plaintext.as_bytes()).map_err(|e| {
                CipherError::EncryptionFailed(format!("failed to write plaintext: {}", e))
            })?;
        }

        // Wait for output
        let output = child
            .wait_with_output()
            .map_err(|e| CipherError::EncryptionFailed(format!("gcloud command failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CipherError::EncryptionFailed(format!(
                "gcloud kms encrypt failed: {}",
                stderr
            ))
            .into());
        }

        // gcloud returns binary ciphertext, encode as base64
        let encoded = ::base64::engine::general_purpose::STANDARD.encode(&output.stdout);

        trace!(ciphertext_len = encoded.len(), "encrypted with GCP KMS");
        Ok(encoded)
    }

    fn decrypt(&self, ciphertext: &str, _identity: &()) -> Result<String> {
        trace!(ciphertext_len = ciphertext.len(), "decrypting with GCP KMS");

        Self::check_gcloud()?;

        // Decode from base64
        use ::base64::Engine;
        let blob = ::base64::engine::general_purpose::STANDARD
            .decode(ciphertext)
            .map_err(|e| CipherError::DecryptionFailed(format!("invalid base64: {}", e)))?;

        let (project, location, keyring, key) = self.parse_resource_name()?;

        // Use gcloud to decrypt
        let mut child = Command::new("gcloud")
            .args([
                "kms",
                "decrypt",
                "--project",
                &project,
                "--location",
                &location,
                "--keyring",
                &keyring,
                "--key",
                &key,
                "--ciphertext-file",
                "-",
                "--plaintext-file",
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| CipherError::DecryptionFailed(format!("failed to spawn gcloud: {}", e)))?;

        // Write ciphertext to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&blob).map_err(|e| {
                CipherError::DecryptionFailed(format!("failed to write ciphertext: {}", e))
            })?;
        }

        // Wait for output
        let output = child
            .wait_with_output()
            .map_err(|e| CipherError::DecryptionFailed(format!("gcloud command failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CipherError::DecryptionFailed(format!(
                "gcloud kms decrypt failed: {}",
                stderr
            ))
            .into());
        }

        // Convert to string
        let plaintext = String::from_utf8(output.stdout)
            .map_err(|e| CipherError::DecryptionFailed(format!("UTF-8 error: {}", e)))?;

        trace!(plaintext_len = plaintext.len(), "decrypted with GCP KMS");
        Ok(plaintext)
    }
}
