//! The primary interface for burrow operations.
//!
//! Vault owns the configuration and provides all secret and team operations.

use crate::core::config::{self, Config};
use crate::core::recipient::Recipient;
use crate::core::store;
use crate::core::types::SecretKey;
use crate::core::{secrets, team};
use crate::error::Result;
use zeroize::Zeroizing;

/// The primary interface for burrow operations.
///
/// Owns the config, manages keys, and provides all secret operations.
/// This is the main entry point for all vault interactions.
#[derive(Debug)]
pub struct Vault {
    config: Config,
    project_id: String,
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

        Ok(Self { config, project_id })
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

        Ok(Self { config, project_id })
    }

    /// Set a secret.
    ///
    /// Encrypts the value for all configured recipients and saves to config.
    ///
    /// # Arguments
    ///
    /// * `key` - Secret key name (must be valid env var name)
    /// * `value` - Plaintext secret value
    /// * `force` - Overwrite if the key already exists
    ///
    /// # Errors
    ///
    /// Returns `ValidationError` if key or value is invalid.
    /// Returns `SecretError::AlreadyExists` if key exists and `force` is false.
    pub fn set(&mut self, key: &str, value: &str, force: bool) -> Result<()> {
        secrets::set(&mut self.config, key, value, force)?;
        Ok(())
    }

    /// Get a decrypted secret.
    ///
    /// # Arguments
    ///
    /// * `key` - Secret key name
    ///
    /// # Returns
    ///
    /// The decrypted plaintext value wrapped in `Zeroizing` for secure memory cleanup.
    ///
    /// # Errors
    ///
    /// Returns `SecretError::NotFound` if the key doesn't exist.
    /// Returns `CipherError` if decryption fails.
    pub fn get(&self, key: &str) -> Result<Zeroizing<String>> {
        secrets::get(&self.config, key)
    }

    /// Remove a secret.
    ///
    /// # Arguments
    ///
    /// * `key` - Secret key name
    ///
    /// # Errors
    ///
    /// Returns `SecretError::NotFound` if the key doesn't exist.
    pub fn remove(&mut self, key: &str) -> Result<()> {
        secrets::remove(&mut self.config, key)?;
        Ok(())
    }

    /// List all secret keys (names only, not values).
    pub fn list(&self) -> Vec<SecretKey> {
        secrets::list(&self.config)
    }

    /// Decrypt all secrets.
    ///
    /// Used for unlock/run/export operations.
    ///
    /// # Returns
    ///
    /// Vector of (key, plaintext_value) pairs with values in `Zeroizing` for secure cleanup.
    ///
    /// # Errors
    ///
    /// Returns error if decryption of any secret fails.
    pub fn decrypt_all(&self) -> Result<Vec<(SecretKey, Zeroizing<String>)>> {
        secrets::decrypt_all(&self.config)
    }

    /// Re-encrypt all secrets (after team changes).
    ///
    /// Decrypts all secrets and re-encrypts them for the current recipient set.
    /// Call this after adding or removing team members.
    ///
    /// # Errors
    ///
    /// Returns error if decryption or re-encryption fails.
    pub fn reencrypt_all(&mut self) -> Result<()> {
        secrets::reencrypt_all(&mut self.config)?;
        Ok(())
    }

    /// Add a team member.
    ///
    /// Validates the public key, adds the recipient to config, and re-encrypts
    /// all secrets so the new member can decrypt them.
    ///
    /// # Arguments
    ///
    /// * `name` - Display name for the team member
    /// * `key` - age public key string
    ///
    /// # Errors
    ///
    /// Returns `CipherError` if the public key is invalid.
    /// Returns error if re-encryption fails.
    pub fn add_recipient(&mut self, name: &str, key: &str) -> Result<()> {
        team::add(&mut self.config, name, key)?;
        Ok(())
    }

    /// Remove a team member.
    ///
    /// Removes the recipient from config and re-encrypts all secrets so the
    /// removed member can no longer decrypt them.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the team member to remove
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::RecipientNotFound` if the member doesn't exist.
    /// Returns error if re-encryption fails.
    pub fn remove_recipient(&mut self, name: &str) -> Result<()> {
        team::remove(&mut self.config, name)?;
        Ok(())
    }

    /// List team members.
    ///
    /// # Returns
    ///
    /// Vector of validated `Recipient` instances.
    pub fn recipients(&self) -> Vec<Recipient> {
        team::list(&self.config)
            .into_iter()
            .filter_map(|(name, key)| Recipient::new(name, key).ok())
            .collect()
    }

    /// Import secrets from .env file.
    ///
    /// Reads key=value pairs from the file and encrypts them.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .env file
    ///
    /// # Returns
    ///
    /// Vector of imported secret keys.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or secrets cannot be encrypted.
    pub fn import(&mut self, path: &str) -> Result<Vec<SecretKey>> {
        crate::core::env::import(&mut self.config, path)
    }

    /// Export as .env format.
    ///
    /// Decrypts all secrets and formats them as KEY=value pairs.
    ///
    /// # Returns
    ///
    /// String in .env format.
    ///
    /// # Errors
    ///
    /// Returns error if decryption fails.
    pub fn export(&self) -> Result<String> {
        crate::core::env::export(&self.config)
    }

    /// Unlock to .env file.
    ///
    /// Decrypts all secrets and writes them to `.env` in the current directory.
    ///
    /// # Returns
    ///
    /// Number of secrets written.
    ///
    /// # Errors
    ///
    /// Returns error if decryption or file write fails.
    pub fn unlock(&self) -> Result<usize> {
        crate::core::env::unlock(&self.config)
    }

    /// Get config reference.
    ///
    /// Provides read-only access to the underlying configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Project ID.
    ///
    /// Returns the unique identifier for this vault, derived from the directory name.
    pub fn project_id(&self) -> &str {
        &self.project_id
    }
}
