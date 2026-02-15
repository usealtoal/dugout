//! Configuration file management.
//!
//! Handles reading, writing, and validating `.dugout.toml` configuration files.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tracing::debug;

use crate::core::constants;
use crate::core::types::{EncryptedValue, MemberName, PublicKey, SecretKey};
use crate::core::vault;
use crate::error::{ConfigError, Result};

/// Project configuration stored in `.dugout.toml`
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Metadata about the vault configuration
    pub dugout: Meta,
    /// Optional KMS configuration for hybrid encryption
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kms: Option<KmsConfig>,
    /// Map of recipient names to age public keys.
    #[serde(default)]
    pub recipients: BTreeMap<MemberName, PublicKey>,
    /// Map of secret keys to their encrypted values
    #[serde(default)]
    pub secrets: BTreeMap<SecretKey, EncryptedValue>,
}

/// KMS configuration for hybrid encryption.
///
/// When present, secrets are encrypted for both age recipients (developers)
/// and a cloud KMS key (production). Provider is auto-detected from key format.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KmsConfig {
    /// KMS key identifier.
    ///
    /// - AWS: `arn:aws:kms:us-east-1:123456789012:key/abc-123`
    /// - GCP: `projects/my-project/locations/global/keyRings/my-ring/cryptoKeys/my-key`
    pub key: String,
}

/// Metadata section of the configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    /// Configuration version
    pub version: String,
    /// SHA-256 hash of sorted recipient public keys (for sync detection)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipients_hash: Option<String>,
}

impl Config {
    /// Create a new empty configuration with current version
    pub fn new() -> Self {
        Self {
            dugout: Meta {
                version: env!("CARGO_PKG_VERSION").to_string(),
                recipients_hash: None,
            },
            kms: None,
            recipients: BTreeMap::new(),
            secrets: BTreeMap::new(),
        }
    }

    /// Path to the configuration file for a given vault.
    pub fn config_path_for(vault: Option<&str>) -> PathBuf {
        constants::vault_path(vault)
    }

    /// Check if a configuration file exists for the given vault.
    pub fn exists_for(vault: Option<&str>) -> bool {
        Self::config_path_for(vault).exists()
    }

    /// Load configuration from vault file.
    pub fn load_from(vault: Option<&str>) -> Result<Self> {
        let path = Self::config_path_for(vault);
        debug!(path = %path.display(), "loading config");

        if !path.exists() {
            return Err(ConfigError::NotInitialized.into());
        }
        let contents = std::fs::read_to_string(&path).map_err(ConfigError::ReadFile)?;
        let config: Self = toml::from_str(&contents).map_err(ConfigError::Parse)?;

        debug!(
            secrets = config.secrets.len(),
            recipients = config.recipients.len(),
            "config loaded"
        );

        config.validate()?;
        Ok(config)
    }

    /// Save configuration to vault file.
    ///
    /// Uses atomic write (temp file + rename) to prevent corruption on crash.
    pub fn save_to(&self, vault: Option<&str>) -> Result<()> {
        debug!("saving config");
        let contents = toml::to_string_pretty(self).map_err(ConfigError::Serialize)?;
        let target_path = Self::config_path_for(vault);

        // Write to temp file in same directory, then rename for atomicity
        let temp_path = target_path.with_extension("toml.tmp");
        std::fs::write(&temp_path, &contents)?;
        std::fs::rename(&temp_path, &target_path)?;

        Ok(())
    }

    /// Path to the default configuration file (backward compat).
    pub fn config_path() -> PathBuf {
        Self::config_path_for(None)
    }

    /// Check if the default configuration file exists (backward compat).
    pub fn exists() -> bool {
        Self::exists_for(None)
    }

    /// Load configuration from default vault (backward compat).
    pub fn load() -> Result<Self> {
        Self::load_from(None)
    }

    /// Save configuration to default vault (backward compat).
    pub fn save(&self) -> Result<()> {
        self.save_to(None)
    }

    /// Unique project identifier based on the current directory name
    pub fn project_id(&self) -> String {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "default".to_string())
    }

    /// Check if KMS hybrid mode is configured.
    pub fn has_kms(&self) -> bool {
        self.kms.is_some()
    }

    /// Get the KMS key if configured.
    pub fn kms_key(&self) -> Option<&str> {
        self.kms.as_ref().map(|k| k.key.as_str())
    }

    /// Validate the configuration structure and contents.
    ///
    /// Checks:
    /// - Version field is valid semver
    /// - At least one recipient exists
    /// - Recipients are valid age public keys
    /// - All secret keys are valid environment variable names
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::InvalidValue` or `ConfigError::MissingField` on validation failure.
    pub fn validate(&self) -> Result<()> {
        use crate::core::cipher;

        debug!("validating config");

        // Check version is valid semver
        if self.dugout.version.is_empty() {
            return Err(ConfigError::MissingField { field: "version" }.into());
        }

        // Try to parse as semver (basic check - just ensure it has valid format)
        let version_parts: Vec<&str> = self.dugout.version.split('.').collect();
        if version_parts.len() < 2 {
            return Err(ConfigError::InvalidValue {
                field: "version",
                reason: format!("not a valid semver: {}", self.dugout.version),
            }
            .into());
        }

        // Check at least one recipient exists
        if self.recipients.is_empty() {
            return Err(ConfigError::NoRecipients.into());
        }

        // Validate recipients are valid age public keys
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
            vault::validate_key(key)?;
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Ensure `.gitignore` contains entries to ignore `.env` files
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

    #[test]
    fn test_config_path_for_vault() {
        assert_eq!(
            Config::config_path_for(None),
            std::path::PathBuf::from(".dugout.toml")
        );
        assert_eq!(
            Config::config_path_for(Some("dev")),
            std::path::PathBuf::from(".dugout.dev.toml")
        );
    }
}
