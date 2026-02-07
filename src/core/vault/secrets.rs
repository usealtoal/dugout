//! Secret operations.
//!
//! CRUD operations for encrypted secrets in the vault.

use super::{get_recipients, validate_key, validate_value, Vault};
use crate::core::cipher;
use crate::core::domain::Secret;
use crate::core::types::SecretKey;
use crate::error::{ConfigError, Result, SecretError};
use zeroize::Zeroizing;

impl Vault {
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
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
