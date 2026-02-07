//! Configuration file management.
//!
//! Handles reading, writing, and validating `.burrow.toml` configuration files.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::error::{ConfigError, Result};

/// Configuration file name.
const CONFIG_FILE: &str = ".burrow.toml";

/// Root configuration structure.
#[derive(Debug, Serialize, Deserialize)]
pub struct BurrowConfig {
    /// Metadata about the burrow configuration.
    pub burrow: BurrowMeta,
    /// Map of recipient names to their age public keys.
    #[serde(default)]
    pub recipients: BTreeMap<String, String>,
    /// Map of secret keys to their encrypted values.
    #[serde(default)]
    pub secrets: BTreeMap<String, String>,
}

/// Burrow metadata section.
#[derive(Debug, Serialize, Deserialize)]
pub struct BurrowMeta {
    /// Configuration version.
    pub version: String,
}

impl BurrowConfig {
    /// Create a new empty configuration.
    pub fn new() -> Self {
        Self {
            burrow: BurrowMeta {
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            recipients: BTreeMap::new(),
            secrets: BTreeMap::new(),
        }
    }

    /// Get the path to the configuration file.
    pub fn config_path() -> PathBuf {
        PathBuf::from(CONFIG_FILE)
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
        if !path.exists() {
            return Err(ConfigError::NotInitialized.into());
        }
        let contents = std::fs::read_to_string(&path)
            .map_err(ConfigError::ReadFile)?;
        let config: Self = toml::from_str(&contents)
            .map_err(ConfigError::Parse)?;
        Ok(config)
    }

    /// Save configuration to `.burrow.toml`.
    ///
    /// # Errors
    ///
    /// Returns error if serialization or file write fails.
    pub fn save(&self) -> Result<()> {
        let contents = toml::to_string_pretty(self)
            .map_err(ConfigError::Serialize)?;
        std::fs::write(Self::config_path(), contents)?;
        Ok(())
    }

    /// Get a unique project identifier based on the current directory name.
    pub fn project_id(&self) -> String {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "default".to_string())
    }
}

impl Default for BurrowConfig {
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
    let entries = [".env", ".env.*", "!.env.example"];

    let existing = if gitignore.exists() {
        std::fs::read_to_string(gitignore)?
    } else {
        String::new()
    };

    let mut updated = existing.clone();
    for entry in entries {
        if !existing.lines().any(|l| l.trim() == entry) {
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
