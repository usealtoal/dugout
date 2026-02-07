//! Secret operations (set, get, remove, list).
//!
//! High-level operations for managing encrypted secrets in the burrow.

use crate::core::config::BurrowConfig;
use crate::core::crypto;
use crate::core::keystore::KeyStore;
use crate::error::{ConfigError, Result, SecretError};

use age::x25519;

/// Get all recipient public keys from config.
///
/// # Errors
///
/// Returns error if any recipient key is invalid.
fn get_recipients(config: &BurrowConfig) -> Result<Vec<x25519::Recipient>> {
    config
        .recipients
        .values()
        .map(|k| crypto::parse_recipient(k))
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
/// Returns `SecretError::AlreadyExists` if key exists and `force` is false.
/// Returns `ConfigError::NoRecipients` if no recipients are configured.
pub fn set_secret(config: &mut BurrowConfig, key: &str, value: &str, force: bool) -> Result<()> {
    if config.secrets.contains_key(key) && !force {
        return Err(SecretError::AlreadyExists(key.to_string()).into());
    }

    let recipients = get_recipients(config)?;
    if recipients.is_empty() {
        return Err(ConfigError::NoRecipients.into());
    }

    let encrypted = crypto::encrypt(value, &recipients)?;
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
/// Returns `CryptoError` if decryption fails.
pub fn get_secret(config: &BurrowConfig, key: &str) -> Result<String> {
    let encrypted = config
        .secrets
        .get(key)
        .ok_or_else(|| SecretError::NotFound(key.to_string()))?;

    let identity = KeyStore::load_identity(&config.project_id())?;
    let plaintext = crypto::decrypt(encrypted, &identity)?;

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
pub fn remove_secret(config: &mut BurrowConfig, key: &str) -> Result<()> {
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
pub fn list_secrets(config: &BurrowConfig) -> Vec<String> {
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
pub fn decrypt_all(config: &BurrowConfig) -> Result<Vec<(String, String)>> {
    let identity = KeyStore::load_identity(&config.project_id())?;

    let mut pairs = Vec::new();
    for (key, encrypted) in &config.secrets {
        let plaintext = crypto::decrypt(encrypted, &identity)?;
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
pub fn reencrypt_all(config: &mut BurrowConfig) -> Result<()> {
    let identity = KeyStore::load_identity(&config.project_id())?;
    let recipients = get_recipients(config)?;

    let mut updated = std::collections::BTreeMap::new();
    for (key, encrypted) in &config.secrets {
        let plaintext = crypto::decrypt(encrypted, &identity)?;
        let reencrypted = crypto::encrypt(&plaintext, &recipients)?;
        updated.insert(key.clone(), reencrypted);
    }

    config.secrets = updated;
    config.save()?;

    Ok(())
}
