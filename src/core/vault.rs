//! Vault.
//!
//! The primary interface for all burrow operations.

use crate::core::cipher;
use crate::core::config::{self, Config};
use crate::core::domain::{Diff, Env, Identity, Recipient, Secret};
use crate::core::store;
use crate::core::types::{MemberName, PublicKey, SecretKey};
use crate::error::{ConfigError, Result, SecretError, ValidationError};
use zeroize::Zeroizing;

/// The primary interface for burrow operations.
///
/// Owns the config, manages keys, and provides all secret operations.
/// This is the main entry point for all vault interactions.
pub struct Vault {
    config: Config,
    project_id: String,
    identity: Identity,
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
    // --- Construction ---
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

    // --- Secrets ---
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
    /// # Returns
    ///
    /// The created Secret.
    ///
    /// # Errors
    ///
    /// Returns `ValidationError` if key or value is invalid.
    /// Returns `SecretError::AlreadyExists` if key exists and `force` is false.
    pub fn set(&mut self, key: &str, value: &str, force: bool) -> Result<Secret> {
        // Validate input
        validate_key(key)?;
        validate_value(key, value)?;

        if self.config.secrets.contains_key(key) && !force {
            return Err(SecretError::AlreadyExists(key.to_string()).into());
        }

        let recipients = get_recipients(&self.config)?;
        if recipients.is_empty() {
            return Err(ConfigError::NoRecipients.into());
        }

        let encrypted = cipher::encrypt(value, &recipients)?;

        self.config
            .secrets
            .insert(key.to_string(), encrypted.clone());
        self.config.save()?;

        Ok(Secret::new(key.to_string(), encrypted))
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
        let encrypted = self.config.secrets.get(key).ok_or_else(|| {
            let available: Vec<String> = self.config.secrets.keys().cloned().collect();
            SecretError::not_found_with_suggestions(key.to_string(), &available)
        })?;

        let plaintext = cipher::decrypt(encrypted, self.identity.as_age())?;

        Ok(Zeroizing::new(plaintext))
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
        if self.config.secrets.remove(key).is_none() {
            let available: Vec<String> = self.config.secrets.keys().cloned().collect();
            return Err(
                SecretError::not_found_with_suggestions(key.to_string(), &available).into(),
            );
        }
        self.config.save()?;
        Ok(())
    }

    /// List all secrets.
    pub fn list(&self) -> Vec<Secret> {
        self.config
            .secrets
            .iter()
            .map(|(key, value)| Secret::new(key.clone(), value.clone()))
            .collect()
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
        let mut pairs = Vec::new();
        for (key, encrypted) in &self.config.secrets {
            let plaintext = cipher::decrypt(encrypted, self.identity.as_age())?;
            pairs.push((key.clone(), Zeroizing::new(plaintext)));
        }

        Ok(pairs)
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
        let recipients = get_recipients(&self.config)?;

        let mut updated = std::collections::BTreeMap::new();
        for (key, encrypted) in &self.config.secrets {
            // Use Zeroizing to ensure plaintext is wiped after re-encryption
            let plaintext = Zeroizing::new(cipher::decrypt(encrypted, self.identity.as_age())?);
            let reencrypted = cipher::encrypt(&plaintext, &recipients)?;
            updated.insert(key.clone(), reencrypted);
        }

        self.config.secrets = updated;
        self.config.save()?;

        Ok(())
    }

    // --- Team ---
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
        // Validate the key format first - this will return a clear error if invalid
        cipher::parse_recipient(key)?;

        self.config
            .recipients
            .insert(name.to_string(), key.to_string());
        self.config.save()?;

        // Re-encrypt all secrets for the new recipient set
        if !self.config.secrets.is_empty() {
            self.reencrypt_all()?;
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
    /// * `name` - Name of the team member to remove
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::RecipientNotFound` if the member doesn't exist.
    /// Returns error if re-encryption fails.
    pub fn remove_recipient(&mut self, name: &str) -> Result<()> {
        if self.config.recipients.remove(name).is_none() {
            return Err(ConfigError::RecipientNotFound(name.to_string()).into());
        }
        self.config.save()?;

        // Re-encrypt all secrets without the removed recipient
        if !self.config.secrets.is_empty() {
            self.reencrypt_all()?;
        }

        Ok(())
    }

    /// List team members.
    ///
    /// # Returns
    ///
    /// Vector of validated `Recipient` instances.
    pub fn recipients(&self) -> Vec<Recipient> {
        list_recipients(&self.config)
            .into_iter()
            .filter_map(|(name, key)| Recipient::new(name, key).ok())
            .collect()
    }

    // --- Lifecycle ---
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
    pub fn import(&mut self, path: impl AsRef<std::path::Path>) -> Result<Vec<SecretKey>> {
        let env = Env::load(path)?;
        let mut imported = Vec::new();

        for (key, value) in env.entries() {
            // Use internal set_secret helper
            set_secret(&mut self.config, key, value, true)?;
            imported.push(key.clone());
        }

        Ok(imported)
    }

    /// Export as .env format.
    ///
    /// Decrypts all secrets and returns them as an Env instance.
    ///
    /// # Returns
    ///
    /// An `Env` containing all decrypted secrets.
    ///
    /// # Errors
    ///
    /// Returns error if decryption fails.
    pub fn export(&self) -> Result<Env> {
        let pairs = self
            .decrypt_all()?
            .into_iter()
            .map(|(k, v)| (k, v.to_string()))
            .collect();

        Ok(Env::from_pairs(pairs, std::path::PathBuf::from(".env")))
    }

    /// Unlock to .env file.
    ///
    /// Decrypts all secrets and writes them to `.env` in the current directory.
    ///
    /// # Returns
    ///
    /// The written `Env` file.
    ///
    /// # Errors
    ///
    /// Returns error if decryption or file write fails.
    pub fn unlock(&self) -> Result<Env> {
        let env = self.export()?;
        env.save()?;
        Ok(env)
    }

    /// Compute diff between vault and .env file.
    ///
    /// Compares the vault's secrets with a .env file.
    ///
    /// # Arguments
    ///
    /// * `env_path` - Path to the .env file (defaults to `.env`)
    ///
    /// # Returns
    ///
    /// A `Diff` showing the comparison.
    ///
    /// # Errors
    ///
    /// Returns error if decryption fails or .env file cannot be read.
    pub fn diff(&self, env_path: impl AsRef<std::path::Path>) -> Result<Diff> {
        let vault_pairs = self
            .decrypt_all()?
            .into_iter()
            .map(|(k, v)| (k, v.to_string()))
            .collect::<Vec<_>>();

        let env_pairs = if env_path.as_ref().exists() {
            let env = Env::load(env_path)?;
            env.entries().to_vec()
        } else {
            Vec::new()
        };

        Ok(Diff::compute(&vault_pairs, &env_pairs))
    }
}

// --- Private helpers ---

/// Validate a secret key name.
///
/// Secret keys must be valid environment variable names:
/// - Only A-Z, 0-9, and underscore
/// - Cannot start with a digit
/// - Cannot be empty
pub(crate) fn validate_key(key: &str) -> Result<()> {
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
fn get_recipients(config: &Config) -> Result<Vec<age::x25519::Recipient>> {
    config
        .recipients
        .values()
        .map(|k| cipher::parse_recipient(k))
        .collect()
}

/// Internal helper: Set (encrypt) a secret value.
///
/// Used by import() to avoid code duplication.
fn set_secret(config: &mut Config, key: &str, value: &str, force: bool) -> Result<()> {
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

/// Internal helper: List all team members.
///
/// Returns vector of (name, public_key) pairs.
fn list_recipients(config: &Config) -> Vec<(MemberName, PublicKey)> {
    config
        .recipients
        .iter()
        .map(|(name, key)| (name.clone(), key.clone()))
        .collect()
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    struct TestContext {
        _tmp: TempDir,
        _original_dir: std::path::PathBuf,
    }

    impl Drop for TestContext {
        fn drop(&mut self) {
            // Restore original directory before tempdir is cleaned up
            let _ = std::env::set_current_dir(&self._original_dir);
        }
    }

    fn setup_test_vault() -> (TestContext, Vault) {
        let tmp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        let vault = Vault::init("alice").unwrap();
        let ctx = TestContext {
            _tmp: tmp,
            _original_dir: original_dir,
        };
        (ctx, vault)
    }

    // --- Secrets tests ---

    #[test]
    fn test_vault_set_and_get() {
        let (_ctx, mut vault) = setup_test_vault();

        vault.set("API_KEY", "secret123", false).unwrap();
        let value = vault.get("API_KEY").unwrap();

        assert_eq!(value.as_str(), "secret123");
    }

    #[test]
    fn test_vault_remove() {
        let (_ctx, mut vault) = setup_test_vault();

        vault.set("TEMP_SECRET", "value", false).unwrap();
        vault.remove("TEMP_SECRET").unwrap();

        // After removal, get should fail
        assert!(vault.get("TEMP_SECRET").is_err());

        // Verify it's not in the list
        let secrets = vault.list();
        let keys: Vec<String> = secrets.iter().map(|s| s.key().to_string()).collect();
        assert!(!keys.contains(&"TEMP_SECRET".to_string()));
    }

    #[test]
    fn test_vault_list() {
        let (_ctx, mut vault) = setup_test_vault();

        vault.set("KEY_ONE", "value1", false).unwrap();
        vault.set("KEY_TWO", "value2", false).unwrap();
        vault.set("KEY_THREE", "value3", false).unwrap();

        let secrets = vault.list();
        assert_eq!(secrets.len(), 3);

        let keys: Vec<String> = secrets.iter().map(|s| s.key().to_string()).collect();
        assert!(keys.contains(&"KEY_ONE".to_string()));
        assert!(keys.contains(&"KEY_TWO".to_string()));
        assert!(keys.contains(&"KEY_THREE".to_string()));
    }

    // --- Team tests ---

    #[test]
    fn test_vault_add_recipient() {
        let (_ctx, mut vault) = setup_test_vault();

        // Set a secret first
        vault.set("SHARED_SECRET", "value", false).unwrap();

        // Generate a second keypair
        let identity = age::x25519::Identity::generate();
        let pubkey = identity.to_public().to_string();

        // Add the new recipient
        vault.add_recipient("bob", &pubkey).unwrap();

        // Verify the recipient was added
        let recipients = vault.recipients();
        assert_eq!(recipients.len(), 2);
        assert!(recipients.iter().any(|r| r.name() == "bob"));

        // Verify the secret can still be decrypted (by alice's key)
        let value = vault.get("SHARED_SECRET").unwrap();
        assert_eq!(value.as_str(), "value");
    }

    #[test]
    fn test_vault_remove_recipient() {
        let (_ctx, mut vault) = setup_test_vault();

        // Add a second recipient
        let identity = age::x25519::Identity::generate();
        let pubkey = identity.to_public().to_string();
        vault.add_recipient("bob", &pubkey).unwrap();

        assert_eq!(vault.recipients().len(), 2);

        // Remove bob
        vault.remove_recipient("bob").unwrap();

        let recipients = vault.recipients();
        assert_eq!(recipients.len(), 1);
        assert!(recipients.iter().all(|r| r.name() != "bob"));
    }

    #[test]
    fn test_vault_reencrypt_after_team_change() {
        let (_ctx, mut vault) = setup_test_vault();

        // Set a secret
        vault.set("TEAM_SECRET", "original", false).unwrap();

        // Add a new member
        let identity = age::x25519::Identity::generate();
        let pubkey = identity.to_public().to_string();
        vault.add_recipient("bob", &pubkey).unwrap();

        // Secret should still decrypt to the same value (using alice's key)
        let value = vault.get("TEAM_SECRET").unwrap();
        assert_eq!(value.as_str(), "original");

        // Verify re-encryption worked - decrypt all should succeed
        let all_secrets = vault.decrypt_all().unwrap();
        assert_eq!(all_secrets.len(), 1);
        assert_eq!(all_secrets[0].0, "TEAM_SECRET");
        assert_eq!(all_secrets[0].1.as_str(), "original");
    }

    // --- Lifecycle tests ---

    #[test]
    fn test_vault_import() {
        let (_ctx, mut vault) = setup_test_vault();

        let env_content = "IMPORT_ONE=value1\nIMPORT_TWO=value2\n";
        fs::write(".env.test", env_content).unwrap();

        let imported = vault.import(".env.test").unwrap();
        assert_eq!(imported.len(), 2);

        assert_eq!(vault.get("IMPORT_ONE").unwrap().as_str(), "value1");
        assert_eq!(vault.get("IMPORT_TWO").unwrap().as_str(), "value2");
    }

    #[test]
    fn test_vault_export_roundtrip() {
        let (_ctx, mut vault) = setup_test_vault();

        vault.set("EXPORT_KEY", "export_value", false).unwrap();
        vault.set("ANOTHER_KEY", "another_value", false).unwrap();

        let env = vault.export().unwrap();
        let exported = format!("{}", env);

        assert!(exported.contains("EXPORT_KEY=export_value"));
        assert!(exported.contains("ANOTHER_KEY=another_value"));
    }
}
