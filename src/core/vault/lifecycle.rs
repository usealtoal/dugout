//! Lifecycle operations.
//!
//! Import, export, unlock, and diff operations for the vault.

use super::{set_secret, Vault};
use crate::core::domain::{Diff, Env};
use crate::core::types::SecretKey;
use crate::error::Result;

impl Vault {
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
