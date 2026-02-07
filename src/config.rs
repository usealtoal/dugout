use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::{BurrowError, Result};

const CONFIG_FILE: &str = ".burrow.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct BurrowConfig {
    pub burrow: BurrowMeta,
    #[serde(default)]
    pub recipients: BTreeMap<String, String>,
    #[serde(default)]
    pub secrets: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BurrowMeta {
    pub version: String,
}

impl BurrowConfig {
    pub fn new() -> Self {
        Self {
            burrow: BurrowMeta {
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            recipients: BTreeMap::new(),
            secrets: BTreeMap::new(),
        }
    }

    pub fn config_path() -> PathBuf {
        PathBuf::from(CONFIG_FILE)
    }

    pub fn exists() -> bool {
        Self::config_path().exists()
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Err(BurrowError::NotInitialized);
        }
        let contents = std::fs::read_to_string(&path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(Self::config_path(), contents)?;
        Ok(())
    }

    pub fn project_id(&self) -> String {
        // Use current directory name as project identifier
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "default".to_string())
    }
}

pub fn ensure_gitignore() -> Result<()> {
    let gitignore = Path::new(".gitignore");
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
