//! The primary interface for burrow operations.
//!
//! Vault owns the configuration and provides all secret and team operations.

mod audit;
mod lifecycle;
mod secrets;
mod team;

use crate::core::cipher;
use crate::core::config::{self, Config};
use crate::core::domain::Identity;
use crate::core::store;
use crate::error::{ConfigError, Result, SecretError, ValidationError};

/// The primary interface for burrow operations.
///
/// Owns the config, manages keys, and provides all secret operations.
/// This is the main entry point for all vault interactions.
pub struct Vault {
    pub(super) config: Config,
    pub(super) project_id: String,
    pub(super) identity: Identity,
}

impl std::fmt::Debug for Vault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vault")
            .field("config", &self.config)
            .field("project_id", &self.project_id)
            .field("identity", &self.identity)
            .finish()
    }
}

impl Vault {
    /// Open an existing vault in the current directory.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::NotInitialized` if no `.burrow.toml` exists.
    /// Returns error if the configuration is invalid or cannot be read.
    pub fn open() -> Result<Self> {
        let config = Config::load()?;
        let project_id = config.project_id();
        let identity = store::load_identity(&project_id)?;

        Ok(Self {
            config,
            project_id,
            identity,
        })
    }

    /// Initialize a new vault.
    ///
    /// Creates a new `.burrow.toml` configuration file, generates a keypair,
    /// and adds the specified user as the first recipient.
    ///
    /// # Arguments
    ///
    /// * `name` - Display name for the initial team member
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::AlreadyInitialized` if vault already exists.
    /// Returns error if keypair generation or file operations fail.
    pub fn init(name: &str) -> Result<Self> {
        if Config::exists() {
            return Err(crate::error::ConfigError::AlreadyInitialized.into());
        }

        let mut config = Config::new();
        let project_id = config.project_id();

        let public_key = store::generate_keypair(&project_id)?;
        config
            .recipients
            .insert(name.to_string(), public_key.clone());
        config.save()?;

        config::ensure_gitignore()?;

        let identity = store::load_identity(&project_id)?;

        Ok(Self {
            config,
            project_id,
            identity,
        })
    }

    /// Get config reference.
    ///
    /// Provides read-only access to the underlying configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the vault's identity.
    pub fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Project ID.
    ///
    /// Returns the unique identifier for this vault, derived from the directory name.
    pub fn project_id(&self) -> &str {
        &self.project_id
    }
}

// Private helper functions shared across modules

/// Validate a secret key name.
///
/// Secret keys must be valid environment variable names:
/// - Only A-Z, 0-9, and underscore
/// - Cannot start with a digit
/// - Cannot be empty
pub(super) fn validate_key(key: &str) -> Result<()> {
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
pub(super) fn validate_value(key: &str, value: &str) -> Result<()> {
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
pub(super) fn get_recipients(config: &Config) -> Result<Vec<age::x25519::Recipient>> {
    config
        .recipients
        .values()
        .map(|k| cipher::parse_recipient(k))
        .collect()
}

/// Internal helper: Set (encrypt) a secret value.
///
/// Used by import() to avoid code duplication.
pub(super) fn set_secret(config: &mut Config, key: &str, value: &str, force: bool) -> Result<()> {
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
