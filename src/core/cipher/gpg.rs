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

#[cfg(all(test, feature = "gpg", feature = "test-gpg"))]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    /// Skip test if GPG is not available
    macro_rules! skip_without_gpg {
        () => {
            if Command::new("gpg").arg("--version").output().is_err() {
                eprintln!("SKIPPED: gpg not installed");
                return;
            }
        };
    }

    /// Set up a temporary GPG home directory with a test key.
    /// Returns (temp_dir, fingerprint, email)
    fn setup_test_gpg_home() -> (TempDir, String, String) {
        let temp = TempDir::new().expect("failed to create temp dir");
        let gpg_home = temp.path().join(".gnupg");
        fs::create_dir_all(&gpg_home).expect("failed to create GPG home");

        // Set GNUPGHOME
        env::set_var("GNUPGHOME", &gpg_home);

        let email = "test@gpg.local";

        // Create key generation batch file
        let key_params = format!(
            r#"%no-protection
Key-Type: RSA
Key-Length: 2048
Name-Real: Test User
Name-Email: {}
Expire-Date: 0
%commit
"#,
            email
        );

        let batch_file = gpg_home.join("gen-key-batch");
        fs::write(&batch_file, key_params).expect("failed to write batch file");

        // Generate the key
        let output = Command::new("gpg")
            .args(["--batch", "--gen-key"])
            .arg(&batch_file)
            .env("GNUPGHOME", &gpg_home)
            .output()
            .expect("failed to generate GPG key");

        if !output.status.success() {
            panic!(
                "GPG key generation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Get the fingerprint
        let output = Command::new("gpg")
            .args(["--list-keys", "--with-colons", email])
            .env("GNUPGHOME", &gpg_home)
            .output()
            .expect("failed to list GPG keys");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let fingerprint = stdout
            .lines()
            .find(|line| line.starts_with("fpr:"))
            .and_then(|line| line.split(':').nth(9))
            .expect("failed to extract fingerprint")
            .to_string();

        (temp, fingerprint, email.to_string())
    }

    /// Generate a second GPG key in the given home directory.
    /// Returns (fingerprint, email)
    fn setup_second_test_key(gpg_home: &std::path::Path) -> (String, String) {
        let email = "bob@gpg.local";

        let key_params = format!(
            r#"%no-protection
Key-Type: RSA
Key-Length: 2048
Name-Real: Bob User
Name-Email: {}
Expire-Date: 0
%commit
"#,
            email
        );

        let batch_file = gpg_home.join("gen-key-batch-2");
        fs::write(&batch_file, key_params).expect("failed to write batch file");

        let output = Command::new("gpg")
            .args(["--batch", "--gen-key"])
            .arg(&batch_file)
            .env("GNUPGHOME", gpg_home)
            .output()
            .expect("failed to generate second GPG key");

        if !output.status.success() {
            panic!(
                "Second GPG key generation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Get the fingerprint
        let output = Command::new("gpg")
            .args(["--list-keys", "--with-colons", email])
            .env("GNUPGHOME", gpg_home)
            .output()
            .expect("failed to list GPG keys");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let fingerprint = stdout
            .lines()
            .find(|line| line.starts_with("fpr:"))
            .and_then(|line| line.split(':').nth(9))
            .expect("failed to extract fingerprint")
            .to_string();

        (fingerprint, email.to_string())
    }

    #[test]
    fn test_gpg_check_available() {
        skip_without_gpg!();
        assert!(Gpg::check_gpg().is_ok());
    }

    #[test]
    fn test_gpg_encrypt_decrypt_roundtrip() {
        skip_without_gpg!();

        let (_temp, _fingerprint, email) = setup_test_gpg_home();
        let gpg = Gpg;

        let plaintext = "super secret data";
        let recipients = vec![email];

        // Encrypt
        let ciphertext = gpg
            .encrypt(plaintext, &recipients)
            .expect("encryption failed");

        // Verify it's armored
        assert!(
            ciphertext.contains("-----BEGIN PGP MESSAGE-----"),
            "should be PGP armored"
        );

        // Decrypt
        let decrypted = gpg.decrypt(&ciphertext, &()).expect("decryption failed");

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_gpg_encrypt_multiple_recipients() {
        skip_without_gpg!();

        let (_temp, _fingerprint, email1) = setup_test_gpg_home();
        let gpg_home = _temp.path().join(".gnupg");
        let (_fingerprint2, email2) = setup_second_test_key(&gpg_home);

        let gpg = Gpg;
        let plaintext = "shared secret";
        let recipients = vec![email1.clone(), email2.clone()];

        // Encrypt for both recipients
        let ciphertext = gpg
            .encrypt(plaintext, &recipients)
            .expect("encryption with multiple recipients failed");

        // Should be able to decrypt with the first key (which is what's active in GNUPGHOME)
        let decrypted = gpg
            .decrypt(&ciphertext, &())
            .expect("decryption with first key failed");

        assert_eq!(decrypted, plaintext);

        // Note: Testing decryption with the second key would require switching GNUPGHOME
        // or exporting/importing the private key, which is complex for this unit test.
        // The fact that encryption succeeded with both recipients is the key test here.
    }

    #[test]
    fn test_gpg_decrypt_wrong_key_fails() {
        skip_without_gpg!();

        // Create first key and encrypt with it
        let (_temp1, _fp1, email1) = setup_test_gpg_home();
        let gpg = Gpg;
        let plaintext = "secret for alice only";

        let ciphertext = gpg
            .encrypt(plaintext, &vec![email1])
            .expect("encryption failed");

        // Create a completely separate GPG home with a different key
        let temp2 = TempDir::new().expect("failed to create second temp dir");
        let gpg_home2 = temp2.path().join(".gnupg");
        fs::create_dir_all(&gpg_home2).expect("failed to create second GPG home");

        // Switch to the new GPG home
        env::set_var("GNUPGHOME", &gpg_home2);

        // Generate a different key
        let email2 = "other@gpg.local";
        let key_params = format!(
            r#"%no-protection
Key-Type: RSA
Key-Length: 2048
Name-Real: Other User
Name-Email: {}
Expire-Date: 0
%commit
"#,
            email2
        );

        let batch_file = gpg_home2.join("gen-key-batch");
        fs::write(&batch_file, key_params).expect("failed to write batch file");

        let output = Command::new("gpg")
            .args(["--batch", "--gen-key"])
            .arg(&batch_file)
            .env("GNUPGHOME", &gpg_home2)
            .output()
            .expect("failed to generate different key");

        assert!(output.status.success());

        // Try to decrypt with the wrong key - should fail
        let result = gpg.decrypt(&ciphertext, &());
        assert!(result.is_err(), "decryption with wrong key should fail");

        // Restore original GNUPGHOME
        env::set_var("GNUPGHOME", _temp1.path().join(".gnupg"));
    }

    #[test]
    fn test_gpg_name() {
        let gpg = Gpg;
        assert_eq!(gpg.name(), "gpg");
    }

    #[test]
    fn test_gpg_encrypt_empty_recipients_fails() {
        skip_without_gpg!();

        let (_temp, _fp, _email) = setup_test_gpg_home();
        let gpg = Gpg;

        let result = gpg.encrypt("test", &vec![]);
        assert!(result.is_err(), "encryption with no recipients should fail");
    }

    #[test]
    fn test_gpg_encrypt_invalid_recipient_fails() {
        skip_without_gpg!();

        let (_temp, _fp, _email) = setup_test_gpg_home();
        let gpg = Gpg;

        let result = gpg.encrypt("test", &vec!["nonexistent@invalid.key".to_string()]);
        assert!(
            result.is_err(),
            "encryption with invalid recipient should fail"
        );
    }

    #[test]
    fn test_gpg_decrypt_invalid_ciphertext_fails() {
        skip_without_gpg!();

        let (_temp, _fp, _email) = setup_test_gpg_home();
        let gpg = Gpg;

        let result = gpg.decrypt("not a valid pgp message", &());
        assert!(result.is_err(), "decryption of invalid data should fail");
    }
}
