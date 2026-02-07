//! Age encryption backend implementation.
//!
//! Provides encryption/decryption using the age format with x25519 keys
//! and ASCII armor encoding.

use std::io::{Read, Write};

use ::age::x25519;

use super::CryptoBackend;
use crate::error::{CryptoError, Result};

/// Age-based cryptographic backend.
///
/// Uses the age encryption format with x25519 keys.
pub struct AgeBackend;

impl CryptoBackend for AgeBackend {
    type Recipient = x25519::Recipient;
    type Identity = x25519::Identity;

    fn encrypt(&self, plaintext: &str, recipients: &[x25519::Recipient]) -> Result<String> {
        let encryptor =
            age::Encryptor::with_recipients(recipients.iter().map(|r| r as &dyn age::Recipient))
                .map_err(|e| CryptoError::EncryptionFailed(format!("{}", e)))?;

        let mut encrypted = Vec::new();
        let mut writer = encryptor
            .wrap_output(age::armor::ArmoredWriter::wrap_output(
                &mut encrypted,
                age::armor::Format::AsciiArmor,
            )?)
            .map_err(|e| CryptoError::EncryptionFailed(format!("{}", e)))?;

        writer.write_all(plaintext.as_bytes())?;
        let armored = writer
            .finish()
            .map_err(|e| CryptoError::EncryptionFailed(format!("{}", e)))?;
        armored
            .finish()
            .map_err(|e| CryptoError::ArmorFailed(format!("{}", e)))?;

        String::from_utf8(encrypted)
            .map_err(|e| CryptoError::EncryptionFailed(format!("UTF-8 error: {}", e)).into())
    }

    fn decrypt(&self, encrypted: &str, identity: &x25519::Identity) -> Result<String> {
        let reader = age::armor::ArmoredReader::new(encrypted.as_bytes());
        let decryptor = age::Decryptor::new(reader)
            .map_err(|e| CryptoError::DecryptionFailed(format!("{}", e)))?;

        let mut decrypted = Vec::new();
        let mut reader = decryptor
            .decrypt(std::iter::once(identity as &dyn age::Identity))
            .map_err(|e| CryptoError::DecryptionFailed(format!("{}", e)))?;

        reader.read_to_end(&mut decrypted)?;

        String::from_utf8(decrypted)
            .map_err(|e| CryptoError::DecryptionFailed(format!("UTF-8 error: {}", e)).into())
    }
}

/// Parse a public key string into an age recipient.
///
/// # Arguments
///
/// * `key` - age public key string (starts with "age1...")
///
/// # Returns
///
/// Parsed `x25519::Recipient` ready for encryption.
///
/// # Errors
///
/// Returns `CryptoError::InvalidPublicKey` if the key format is invalid.
pub fn parse_recipient(key: &str) -> Result<x25519::Recipient> {
    key.parse::<x25519::Recipient>()
        .map_err(|_| CryptoError::InvalidPublicKey(key.to_string()).into())
}
