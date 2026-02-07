use std::io::{Read, Write};

use age::x25519;

use crate::error::{BurrowError, Result};

/// Encrypt a plaintext string for multiple recipients
pub fn encrypt(plaintext: &str, recipients: &[x25519::Recipient]) -> Result<String> {
    let encryptor = age::Encryptor::with_recipients(
        recipients.iter().map(|r| r as &dyn age::Recipient),
    )
    .map_err(|e| BurrowError::EncryptionFailed(format!("{}", e)))?;

    let mut encrypted = Vec::new();
    let mut writer = encryptor
        .wrap_output(age::armor::ArmoredWriter::wrap_output(
            &mut encrypted,
            age::armor::Format::AsciiArmor,
        )?)
        .map_err(|e| BurrowError::EncryptionFailed(format!("{}", e)))?;

    writer.write_all(plaintext.as_bytes())?;
    let armored = writer
        .finish()
        .map_err(|e| BurrowError::EncryptionFailed(format!("{}", e)))?;
    armored
        .finish()
        .map_err(|e| BurrowError::EncryptionFailed(format!("{}", e)))?;

    let encoded = String::from_utf8(encrypted)
        .map_err(|e| BurrowError::EncryptionFailed(format!("{}", e)))?;

    Ok(encoded)
}

/// Decrypt an encrypted string with a private key
pub fn decrypt(encrypted: &str, identity: &x25519::Identity) -> Result<String> {
    let reader = age::armor::ArmoredReader::new(encrypted.as_bytes());
    let decryptor = age::Decryptor::new(reader)
        .map_err(|e| BurrowError::DecryptionFailed(format!("{}", e)))?;

    let mut decrypted = Vec::new();
    let mut reader = decryptor
        .decrypt(std::iter::once(identity as &dyn age::Identity))
        .map_err(|e| BurrowError::DecryptionFailed(format!("{}", e)))?;

    reader.read_to_end(&mut decrypted)?;

    String::from_utf8(decrypted)
        .map_err(|e| BurrowError::DecryptionFailed(format!("{}", e)))
}

/// Parse a public key string into an age recipient
pub fn parse_recipient(key: &str) -> Result<x25519::Recipient> {
    key.parse::<x25519::Recipient>()
        .map_err(|_| BurrowError::InvalidKey(format!("invalid age public key: {}", key)))
}
