//! Team member management.
//!
//! Operations for adding and removing recipients (team members) who can
//! decrypt secrets.

use crate::core::cipher;
use crate::core::config::Config;
use crate::core::secrets;
use crate::error::{ConfigError, Result};

/// Add a team member by their age public key.
///
/// Validates the public key, adds the recipient to config, and re-encrypts
/// all secrets so the new member can decrypt them.
///
/// # Arguments
///
/// * `config` - Mutable reference to configuration
/// * `name` - Display name for the team member
/// * `public_key` - age public key string
///
/// # Errors
///
/// Returns `CipherError` if the public key is invalid.
/// Returns error if re-encryption fails.
pub fn add(config: &mut Config, name: &str, public_key: &str) -> Result<()> {
    // Validate the key format first - this will return a clear error if invalid
    cipher::parse_recipient(public_key)?;

    config
        .recipients
        .insert(name.to_string(), public_key.to_string());
    config.save()?;

    // Re-encrypt all secrets for the new recipient set
    if !config.secrets.is_empty() {
        secrets::reencrypt_all(config)?;
    }

    Ok(())
}

/// Remove a team member.
///
/// Removes the recipient from config and re-encrypts all secrets so the
/// removed member can no longer decrypt them.
///
/// # Arguments
///
/// * `config` - Mutable reference to configuration
/// * `name` - Name of the team member to remove
///
/// # Errors
///
/// Returns `ConfigError::RecipientNotFound` if the member doesn't exist.
/// Returns error if re-encryption fails.
pub fn remove(config: &mut Config, name: &str) -> Result<()> {
    if config.recipients.remove(name).is_none() {
        return Err(ConfigError::RecipientNotFound(name.to_string()).into());
    }
    config.save()?;

    // Re-encrypt all secrets without the removed recipient
    if !config.secrets.is_empty() {
        secrets::reencrypt_all(config)?;
    }

    Ok(())
}

/// List all team members.
///
/// # Arguments
///
/// * `config` - Configuration reference
///
/// # Returns
///
/// Vector of (name, public_key) pairs.
pub fn list(config: &Config) -> Vec<(String, String)> {
    config
        .recipients
        .iter()
        .map(|(name, key)| (name.clone(), key.clone()))
        .collect()
}
