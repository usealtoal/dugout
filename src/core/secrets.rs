//! Secret operations (set, get, remove, list).
//!
//! High-level operations for managing encrypted secrets in the burrow.

use crate::core::config::Config;
use crate::core::{cipher, store};
use crate::error::{ConfigError, Result, SecretError, ValidationError};

use age::x25519;

/// Validate a secret key name.
///
/// Secret keys must be valid environment variable names:
/// - Only A-Z, 0-9, and underscore
/// - Cannot start with a digit
/// - Cannot be empty
fn validate_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(ValidationError::EmptyKey.into());
    }

    if let Some(first_char) = key.chars().next() {
        if first_char.is_ascii_digit() {
            return Err(ValidationError::InvalidKey {
                key: key.to_string(),
                reason: "cannot start with a digit".to_string(),
            }
            .into());
        }
    }

    for (i, ch) in key.chars().enumerate() {
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            return Err(ValidationError::InvalidKey {
                key: key.to_string(),
                reason: format!(
                    "invalid character '{}' at position {}. Only A-Z, 0-9, and underscore are allowed",
                    ch, i + 1
                ),
            }
            .into());
        }
    }

    Ok(())
}

/// Validate a secret value.
///
/// Secret values cannot be empty.
fn validate_value(key: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(ValidationError::EmptyValue(key.to_string()).into());
    }

    Ok(())
}

/// Get all recipient public keys from config.
///
/// # Errors
///
/// Returns error if any recipient key is invalid.
fn get_recipients(config: &Config) -> Result<Vec<x25519::Recipient>> {
    config
        .recipients
        .values()
        .map(|k| cipher::parse_recipient(k))
        .collect()
}

/// Set (encrypt) a secret value.
///
/// # Arguments
///
/// * `config` - Mutable reference to configuration
/// * `key` - Secret key name (e.g., "DATABASE_URL")
/// * `value` - Plaintext secret value
/// * `force` - Overwrite if the key already exists
///
/// # Errors
///
/// Returns `ValidationError` if key or value is invalid.
/// Returns `SecretError::AlreadyExists` if key exists and `force` is false.
/// Returns `ConfigError::NoRecipients` if no recipients are configured.
pub fn set(config: &mut Config, key: &str, value: &str, force: bool) -> Result<()> {
    // Validate input
    validate_key(key)?;
    validate_value(key, value)?;

    if config.secrets.contains_key(key) && !force {
        return Err(SecretError::AlreadyExists(key.to_string()).into());
    }

    let recipients = get_recipients(config)?;
    if recipients.is_empty() {
        return Err(ConfigError::NoRecipients.into());
    }

    let encrypted = cipher::encrypt(value, &recipients)?;
    config.secrets.insert(key.to_string(), encrypted);
    config.save()?;

    Ok(())
}

/// Get (decrypt) a secret value.
///
/// # Arguments
///
/// * `config` - Configuration reference
/// * `key` - Secret key name
///
/// # Returns
///
/// The decrypted plaintext value.
///
/// # Errors
///
/// Returns `SecretError::NotFound` if the key doesn't exist.
/// Returns `CipherError` if decryption fails.
pub fn get(config: &Config, key: &str) -> Result<String> {
    let encrypted = config
        .secrets
        .get(key)
        .ok_or_else(|| SecretError::NotFound(key.to_string()))?;

    let identity = store::load_identity(&config.project_id())?;
    let plaintext = cipher::decrypt(encrypted, &identity)?;

    Ok(plaintext)
}

/// Remove a secret.
///
/// # Arguments
///
/// * `config` - Mutable reference to configuration
/// * `key` - Secret key name
///
/// # Errors
///
/// Returns `SecretError::NotFound` if the key doesn't exist.
pub fn remove(config: &mut Config, key: &str) -> Result<()> {
    if config.secrets.remove(key).is_none() {
        return Err(SecretError::NotFound(key.to_string()).into());
    }
    config.save()?;
    Ok(())
}

/// List all secret keys (names only, not values).
///
/// # Arguments
///
/// * `config` - Configuration reference
///
/// # Returns
///
/// Vector of secret key names.
pub fn list(config: &Config) -> Vec<String> {
    config.secrets.keys().cloned().collect()
}

/// Decrypt all secrets (for unlock/run operations).
///
/// # Arguments
///
/// * `config` - Configuration reference
///
/// # Returns
///
/// Vector of (key, plaintext_value) pairs.
///
/// # Errors
///
/// Returns error if decryption of any secret fails.
pub fn decrypt_all(config: &Config) -> Result<Vec<(String, String)>> {
    let identity = store::load_identity(&config.project_id())?;

    let mut pairs = Vec::new();
    for (key, encrypted) in &config.secrets {
        let plaintext = cipher::decrypt(encrypted, &identity)?;
        pairs.push((key.clone(), plaintext));
    }

    Ok(pairs)
}

/// Re-encrypt all secrets with updated recipient list.
///
/// Used after adding or removing team members to ensure all secrets
/// are encrypted for the current recipient set.
///
/// # Arguments
///
/// * `config` - Mutable reference to configuration
///
/// # Errors
///
/// Returns error if decryption or re-encryption fails.
pub fn reencrypt_all(config: &mut Config) -> Result<()> {
    let identity = store::load_identity(&config.project_id())?;
    let recipients = get_recipients(config)?;

    let mut updated = std::collections::BTreeMap::new();
    for (key, encrypted) in &config.secrets {
        let plaintext = cipher::decrypt(encrypted, &identity)?;
        let reencrypted = cipher::encrypt(&plaintext, &recipients)?;
        updated.insert(key.clone(), reencrypted);
    }

    config.secrets = updated;
    config.save()?;

    Ok(())
}
