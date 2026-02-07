//! Configuration file management.
//!
//! Handles reading, writing, and validating `.burrow.toml` configuration files.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tracing::debug;

use crate::core::constants;
use crate::core::secrets;
use crate::core::types::{EncryptedValue, MemberName, PublicKey, SecretKey};
use crate::error::{ConfigError, Result};

/// Root configuration structure.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Metadata about the burrow configuration.
    pub burrow: Meta,
    /// Map of recipient names to their age public keys.
    #[serde(default)]
    pub recipients: BTreeMap<MemberName, PublicKey>,
    /// Map of secret keys to their encrypted values.
    #[serde(default)]
    pub secrets: BTreeMap<SecretKey, EncryptedValue>,
}

/// Burrow metadata section.
#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    /// Configuration version.
    pub version: String,
}

impl Config {
    /// Create a new empty configuration.
    pub fn new() -> Self {
        Self {
            burrow: Meta {
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            recipients: BTreeMap::new(),
            secrets: BTreeMap::new(),
        }
    }

    /// Get the path to the configuration file.
    pub fn config_path() -> PathBuf {
        PathBuf::from(constants::CONFIG_FILE)
    }

    /// Check if a configuration file exists in the current directory.
    pub fn exists() -> bool {
        Self::config_path().exists()
    }

    /// Load configuration from `.burrow.toml`.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::NotInitialized` if the file doesn't exist,
    /// or `ConfigError::Parse` if the TOML is malformed.
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        debug!("Loading configuration from {}", path.display());

        if !path.exists() {
            return Err(ConfigError::NotInitialized.into());
        }
        let contents = std::fs::read_to_string(&path).map_err(ConfigError::ReadFile)?;
        let config: Self = toml::from_str(&contents).map_err(ConfigError::Parse)?;

        debug!(
            "Loaded config: {} recipient(s), {} secret(s)",
            config.recipients.len(),
            config.secrets.len()
        );

        // Validate the loaded configuration
        config.validate()?;

        Ok(config)
    }

    /// Save configuration to `.burrow.toml`.
    ///
    /// # Errors
    ///
    /// Returns error if serialization or file write fails.
    pub fn save(&self) -> Result<()> {
        debug!(
            "Saving configuration: {} recipient(s), {} secret(s)",
            self.recipients.len(),
            self.secrets.len()
        );

        let contents = toml::to_string_pretty(self).map_err(ConfigError::Serialize)?;
        std::fs::write(Self::config_path(), contents)?;

        debug!("Configuration saved to .burrow.toml");
        Ok(())
    }

    /// Get a unique project identifier based on the current directory name.
    pub fn project_id(&self) -> String {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "default".to_string())
    }

    /// Validate the configuration structure and contents.
    ///
    /// Checks:
    /// - Version field is valid semver
    /// - At least one recipient exists
    /// - All recipient keys are valid age public keys
    /// - All secret keys are valid environment variable names
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::InvalidValue` or `ConfigError::MissingField` on validation failure.
    pub fn validate(&self) -> Result<()> {
        use crate::core::cipher;

        // Check version is valid semver
        if self.burrow.version.is_empty() {
            return Err(ConfigError::MissingField { field: "version" }.into());
        }

        // Try to parse as semver (basic check - just ensure it has valid format)
        let version_parts: Vec<&str> = self.burrow.version.split('.').collect();
        if version_parts.len() < 2 {
            return Err(ConfigError::InvalidValue {
                field: "version",
                reason: format!("not a valid semver: {}", self.burrow.version),
            }
            .into());
        }

        // Check at least one recipient exists
        if self.recipients.is_empty() {
            return Err(ConfigError::NoRecipients.into());
        }

        // Validate all recipient public keys
        for (name, key) in &self.recipients {
            if cipher::parse_recipient(key).is_err() {
                return Err(ConfigError::InvalidValue {
                    field: "recipients",
                    reason: format!("invalid age public key for recipient '{}': {}", name, key),
                }
                .into());
            }
        }

        // Validate secret keys are valid env var names
        for key in self.secrets.keys() {
            secrets::validate_key(key)?;
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Ensure `.gitignore` contains entries to ignore `.env` files.
///
/// Adds `.env`, `.env.*`, and `!.env.example` if not already present.
///
/// # Errors
///
/// Returns error if file operations fail.
pub fn ensure_gitignore() -> Result<()> {
    let gitignore = std::path::Path::new(".gitignore");

    let existing = if gitignore.exists() {
        std::fs::read_to_string(gitignore)?
    } else {
        String::new()
    };

    let mut updated = existing.clone();
    for entry in constants::GITIGNORE_ENTRIES {
        if !existing.lines().any(|l| l.trim() == *entry) {
            if !updated.is_empty() && !updated.ends_with('\n') {
                updated.push('\n');
            }
            updated.push_str(entry);
            updated.push('\n');
        }
    }

    if updated != existing {
        std::fs::write(gitignore, updated)?;
    }

    Ok(())
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

    fn setup_test_dir() -> TestContext {
        let tmp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        TestContext {
            _tmp: tmp,
            _original_dir: original_dir,
        }
    }

    #[test]
    fn test_config_save_load_roundtrip() {
        let _ctx = setup_test_dir();

        let identity = age::x25519::Identity::generate();
        let pubkey = identity.to_public().to_string();

        let mut config = Config::new();
        config.recipients.insert("alice".to_string(), pubkey);
        config.secrets.insert(
            "TEST_KEY".to_string(),
            "-----BEGIN AGE ENCRYPTED FILE-----\ntest\n-----END AGE ENCRYPTED FILE-----"
                .to_string(),
        );

        config.save().unwrap();
        assert!(Config::exists());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.recipients.len(), 1);
        assert_eq!(loaded.secrets.len(), 1);
        assert!(loaded.recipients.contains_key("alice"));
        assert!(loaded.secrets.contains_key("TEST_KEY"));
    }

    #[test]
    fn test_config_validate_valid() {
        let _ctx = setup_test_dir();

        let identity = age::x25519::Identity::generate();
        let pubkey = identity.to_public().to_string();

        let mut config = Config::new();
        config.recipients.insert("alice".to_string(), pubkey);

        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validate_missing_recipients() {
        let _ctx = setup_test_dir();

        let config = Config::new();
        let result = config.validate();

        assert!(result.is_err());
        // Should fail because no recipients
    }

    #[test]
    fn test_config_validate_bad_secret_key() {
        let _ctx = setup_test_dir();

        let identity = age::x25519::Identity::generate();
        let pubkey = identity.to_public().to_string();

        let mut config = Config::new();
        config.recipients.insert("alice".to_string(), pubkey);
        config.secrets.insert(
            "invalid-key-name".to_string(),
            "encrypted_value".to_string(),
        );

        let result = config.validate();
        assert!(result.is_err());
        // Should fail because secret key has invalid characters
    }
}
