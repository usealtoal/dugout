//! Age encryption backend implementation.
//!
//! Provides encryption/decryption using the age format with x25519 keys
//! and ASCII armor encoding.

use std::io::{Read, Write};

use ::age::x25519;
use tracing::trace;

use super::Cipher;
use crate::error::{CipherError, Result};

/// Age-based cryptographic backend using x25519 keys
pub struct Age;

impl Cipher for Age {
    type Recipient = x25519::Recipient;
    type Identity = x25519::Identity;

    fn name(&self) -> &'static str {
        "age"
    }

    fn encrypt(&self, plaintext: &str, recipients: &[x25519::Recipient]) -> Result<String> {
        trace!(
            recipients = recipients.len(),
            plaintext_len = plaintext.len(),
            "encrypting"
        );

        let encryptor =
            age::Encryptor::with_recipients(recipients.iter().map(|r| r as &dyn age::Recipient))
                .map_err(|e| CipherError::EncryptionFailed(format!("{}", e)))?;

        let mut encrypted = Vec::new();
        let mut writer = encryptor
            .wrap_output(age::armor::ArmoredWriter::wrap_output(
                &mut encrypted,
                age::armor::Format::AsciiArmor,
            )?)
            .map_err(|e| CipherError::EncryptionFailed(format!("{}", e)))?;

        writer.write_all(plaintext.as_bytes())?;
        let armored = writer
            .finish()
            .map_err(|e| CipherError::EncryptionFailed(format!("{}", e)))?;
        armored
            .finish()
            .map_err(|e| CipherError::ArmorFailed(format!("{}", e)))?;

        trace!(ciphertext_len = encrypted.len(), "encrypted");

        String::from_utf8(encrypted)
            .map_err(|e| CipherError::EncryptionFailed(format!("UTF-8 error: {}", e)).into())
    }

    fn decrypt(&self, encrypted: &str, identity: &x25519::Identity) -> Result<String> {
        trace!(ciphertext_len = encrypted.len(), "decrypting");

        let reader = age::armor::ArmoredReader::new(encrypted.as_bytes());
        let decryptor = age::Decryptor::new(reader)
            .map_err(|e| CipherError::DecryptionFailed(format!("{}", e)))?;

        let mut decrypted = Vec::new();
        let mut reader = decryptor
            .decrypt(std::iter::once(identity as &dyn age::Identity))
            .map_err(|e| CipherError::DecryptionFailed(format!("{}", e)))?;

        reader.read_to_end(&mut decrypted)?;

        trace!(plaintext_len = decrypted.len(), "decrypted");

        String::from_utf8(decrypted)
            .map_err(|e| CipherError::DecryptionFailed(format!("UTF-8 error: {}", e)).into())
    }
}

/// Parse a public key string into an age recipient
///
/// # Errors
///
/// Returns `CipherError::InvalidPublicKey` if the key format is invalid.
pub fn parse_recipient(key: &str) -> Result<x25519::Recipient> {
    key.parse::<x25519::Recipient>()
        .map_err(|_| CipherError::InvalidPublicKey(key.to_string()).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let cipher = Age;
        let identity = x25519::Identity::generate();
        let recipient = identity.to_public();

        let plaintext = "Hello, World!";
        let encrypted = cipher.encrypt(plaintext, &[recipient]).unwrap();

        assert_ne!(encrypted, plaintext);
        assert!(encrypted.contains("-----BEGIN AGE ENCRYPTED FILE-----"));

        let decrypted = cipher.decrypt(&encrypted, &identity).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_large_payload() {
        let cipher = Age;
        let identity = x25519::Identity::generate();
        let recipient = identity.to_public();

        // Create a large payload (10KB)
        let plaintext = "A".repeat(10_000);
        let encrypted = cipher.encrypt(&plaintext, &[recipient]).unwrap();

        let decrypted = cipher.decrypt(&encrypted, &identity).unwrap();
        assert_eq!(decrypted, plaintext);
        assert_eq!(decrypted.len(), 10_000);
    }

    #[test]
    fn test_encrypt_with_multiple_recipients() {
        let cipher = Age;

        let identity1 = x25519::Identity::generate();
        let identity2 = x25519::Identity::generate();
        let recipient1 = identity1.to_public();
        let recipient2 = identity2.to_public();

        let plaintext = "Shared secret";
        let encrypted = cipher
            .encrypt(plaintext, &[recipient1, recipient2])
            .unwrap();

        // Both identities should be able to decrypt
        let decrypted1 = cipher.decrypt(&encrypted, &identity1).unwrap();
        assert_eq!(decrypted1, plaintext);

        let decrypted2 = cipher.decrypt(&encrypted, &identity2).unwrap();
        assert_eq!(decrypted2, plaintext);
    }
}
