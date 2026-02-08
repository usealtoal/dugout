//! GPG cipher backend.
//!
//! Encrypts secrets using GnuPG (GNU Privacy Guard).
//! Enable with `--features gpg`.
//!
//! ## Requirements
//!
//! - `gpg` CLI must be installed
//! - GPG keyring must be configured with recipient public keys
//! - Private key must be available for decryption
//!
//! ## Usage
//!
//! Configure your vault with:
//! ```toml
//! [meta]
//! cipher = "gpg"
//!
//! # Recipients are GPG key fingerprints or email addresses
//! [recipients]
//! alice = "alice@example.com"
//! bob = "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234"
//! ```

use std::io::Write;
use std::process::{Command, Stdio};
use tracing::trace;

use super::Cipher;
use crate::error::{CipherError, Result};

/// GPG cipher backend using gpg CLI
#[cfg(feature = "gpg")]
#[allow(dead_code)]
pub struct Gpg;

#[cfg(feature = "gpg")]
impl Gpg {
    /// Check if gpg CLI is available
    #[allow(dead_code)]
    fn check_gpg() -> Result<()> {
        Command::new("gpg")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|_| {
                CipherError::EncryptionFailed(
                    "gpg CLI not found. Install GnuPG from https://gnupg.org/download/".to_string(),
                )
            })?;
        Ok(())
    }
}

#[cfg(feature = "gpg")]
impl Cipher for Gpg {
    // GPG recipients can be email addresses or key fingerprints
    type Recipient = String;
    // GPG doesn't require explicit identity - it uses the keyring
    type Identity = ();

    fn name(&self) -> &'static str {
        "gpg"
    }

    fn encrypt(&self, plaintext: &str, recipients: &[String]) -> Result<String> {
        trace!(
            recipients = recipients.len(),
            plaintext_len = plaintext.len(),
            "encrypting with GPG"
        );

        Self::check_gpg()?;

        if recipients.is_empty() {
            return Err(CipherError::EncryptionFailed("no recipients provided".to_string()).into());
        }

        // Build gpg command
        let mut cmd = Command::new("gpg");
        cmd.args([
            "--encrypt",
            "--armor",
            "--trust-model",
            "always",  // Trust all keys without confirmation
            "--batch", // Non-interactive mode
            "--yes",   // Assume yes to all questions
        ]);

        // Add all recipients
        for recipient in recipients {
            cmd.args(["--recipient", recipient]);
        }

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Spawn the process
        let mut child = cmd
            .spawn()
            .map_err(|e| CipherError::EncryptionFailed(format!("failed to spawn gpg: {}", e)))?;

        // Write plaintext to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(plaintext.as_bytes()).map_err(|e| {
                CipherError::EncryptionFailed(format!("failed to write plaintext: {}", e))
            })?;
        }

        // Wait for output
        let output = child
            .wait_with_output()
            .map_err(|e| CipherError::EncryptionFailed(format!("gpg command failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(
                CipherError::EncryptionFailed(format!("gpg encrypt failed: {}", stderr)).into(),
            );
        }

        // Convert to string (GPG armor is ASCII)
        let ciphertext = String::from_utf8(output.stdout)
            .map_err(|e| CipherError::EncryptionFailed(format!("UTF-8 error: {}", e)))?;

        trace!(ciphertext_len = ciphertext.len(), "encrypted with GPG");
        Ok(ciphertext)
    }

    fn decrypt(&self, ciphertext: &str, _identity: &()) -> Result<String> {
        trace!(ciphertext_len = ciphertext.len(), "decrypting with GPG");

        Self::check_gpg()?;

        // Build gpg command
        let mut cmd = Command::new("gpg");
        cmd.args([
            "--decrypt",
            "--batch", // Non-interactive mode
            "--yes",   // Assume yes to all questions
            "--quiet", // Minimize output
        ]);

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Spawn the process
        let mut child = cmd
            .spawn()
            .map_err(|e| CipherError::DecryptionFailed(format!("failed to spawn gpg: {}", e)))?;

        // Write ciphertext to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(ciphertext.as_bytes()).map_err(|e| {
                CipherError::DecryptionFailed(format!("failed to write ciphertext: {}", e))
            })?;
        }

        // Wait for output
        let output = child
            .wait_with_output()
            .map_err(|e| CipherError::DecryptionFailed(format!("gpg command failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CipherError::DecryptionFailed(format!(
                "gpg decrypt failed: {}. Ensure you have the private key in your keyring.",
                stderr
            ))
            .into());
        }

        // Convert to string
        let plaintext = String::from_utf8(output.stdout)
            .map_err(|e| CipherError::DecryptionFailed(format!("UTF-8 error: {}", e)))?;

        trace!(plaintext_len = plaintext.len(), "decrypted with GPG");
        Ok(plaintext)
    }
}

#[cfg(all(test, feature = "gpg"))]
mod tests {
    use super::*;

    #[test]
    fn test_check_gpg() {
        // This test will fail if gpg is not installed
        // Skip in CI if gpg is not available
        if Command::new("gpg")
            .arg("--version")
            .stdout(Stdio::null())
            .status()
            .is_err()
        {
            eprintln!("Skipping GPG tests - gpg not installed");
            return;
        }

        assert!(Gpg::check_gpg().is_ok());
    }
}
