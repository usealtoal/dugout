//! Team operations.
//!
//! Manage team members who can decrypt vault secrets.

use super::Vault;
use crate::core::cipher;
use crate::core::domain::Recipient;
use crate::core::types::{MemberName, PublicKey};
use crate::error::{ConfigError, Result};

impl Vault {
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
}

/// Internal helper: List all team members.
///
/// Returns vector of (name, public_key) pairs.
fn list_recipients(config: &crate::core::config::Config) -> Vec<(MemberName, PublicKey)> {
    config
        .recipients
        .iter()
        .map(|(name, key)| (name.clone(), key.clone()))
        .collect()
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
}
