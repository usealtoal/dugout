//! Env type.
//!
//! Represents a parsed .env file with typed access.

use crate::error::Result;
use std::path::{Path, PathBuf};

/// A parsed .env file.
#[derive(Debug, Clone)]
pub struct Env {
    entries: Vec<(String, String)>,
    path: PathBuf,
}

impl Env {
    /// Parse an .env file from disk.
    ///
    /// Skips empty lines and comments (lines starting with #).
    /// Supports values with or without quotes.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .env file
    ///
    /// # Errors
    ///
    /// Returns error if the file cannot be read.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)?;
        let mut entries = Vec::new();

        for line in contents.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                entries.push((key, value));
            }
        }

        Ok(Self {
            entries,
            path: path.to_path_buf(),
        })
    }

    /// Create from raw key-value pairs.
    ///
    /// # Arguments
    ///
    /// * `pairs` - Vector of (key, value) tuples
    /// * `path` - Path where this env file would be written
    pub fn from_pairs(pairs: Vec<(String, String)>, path: PathBuf) -> Self {
        Self {
            entries: pairs,
            path,
        }
    }

    /// Write the env file to disk.
    ///
    /// Writes all entries in KEY=value format to the configured path.
    /// Quotes values containing spaces, equals signs, or hash marks.
    ///
    /// # Errors
    ///
    /// Returns error if the file cannot be written.
    pub fn save(&self) -> Result<()> {
        let content = self.to_env_string();
        std::fs::write(&self.path, content)?;
        Ok(())
    }

    /// Get a value by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The environment variable name
    ///
    /// # Returns
    ///
    /// The value if the key exists, `None` otherwise.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// All entries as key-value pairs.
    pub fn entries(&self) -> &[(String, String)] {
        &self.entries
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The file path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Serialize to .env format string.
    ///
    /// Quotes values that contain spaces or special characters.
    fn to_env_string(&self) -> String {
        let mut output = String::new();

        for (key, value) in &self.entries {
            // Quote values that contain spaces or special chars
            if value.contains(' ') || value.contains('#') || value.contains('=') {
                output.push_str(&format!("{}=\"{}\"\n", key, value));
            } else {
                output.push_str(&format!("{}={}\n", key, value));
            }
        }

        output
    }
}

impl std::fmt::Display for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_env_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_env_load_and_entries() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".env");

        let content = "API_KEY=secret123\nDB_URL=postgres://localhost/db\n";
        fs::write(&path, content).unwrap();

        let env = Env::load(&path).unwrap();

        assert_eq!(env.len(), 2);
        assert!(!env.is_empty());
        assert_eq!(env.entries().len(), 2);
        assert_eq!(env.path(), path.as_path());
    }

    #[test]
    fn test_env_from_pairs() {
        let pairs = vec![
            ("KEY1".to_string(), "value1".to_string()),
            ("KEY2".to_string(), "value2".to_string()),
        ];
        let path = PathBuf::from(".env");

        let env = Env::from_pairs(pairs, path.clone());

        assert_eq!(env.len(), 2);
        assert_eq!(env.path(), path.as_path());
    }

    #[test]
    fn test_env_get() {
        let pairs = vec![
            ("API_KEY".to_string(), "secret123".to_string()),
            ("DB_URL".to_string(), "postgres://".to_string()),
        ];
        let env = Env::from_pairs(pairs, PathBuf::from(".env"));

        assert_eq!(env.get("API_KEY"), Some("secret123"));
        assert_eq!(env.get("DB_URL"), Some("postgres://"));
        assert_eq!(env.get("NONEXISTENT"), None);
    }

    #[test]
    fn test_env_display() {
        let pairs = vec![
            ("SIMPLE".to_string(), "value".to_string()),
            ("WITH_SPACE".to_string(), "value with spaces".to_string()),
        ];
        let env = Env::from_pairs(pairs, PathBuf::from(".env"));

        let output = format!("{}", env);

        assert!(output.contains("SIMPLE=value\n"));
        assert!(output.contains("WITH_SPACE=\"value with spaces\"\n"));
    }

    #[test]
    fn test_env_handles_comments() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".env");

        let content =
            "# This is a comment\nAPI_KEY=secret\n# Another comment\nDB_URL=postgres://\n";
        fs::write(&path, content).unwrap();

        let env = Env::load(&path).unwrap();

        // Comments should be skipped
        assert_eq!(env.len(), 2);
        assert_eq!(env.get("API_KEY"), Some("secret"));
        assert_eq!(env.get("DB_URL"), Some("postgres://"));
    }

    #[test]
    fn test_env_handles_quotes() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".env");

        let content = "QUOTED=\"value in quotes\"\nSINGLE='single quotes'\nNONE=no quotes\n";
        fs::write(&path, content).unwrap();

        let env = Env::load(&path).unwrap();

        // Quotes should be stripped during parsing
        assert_eq!(env.get("QUOTED"), Some("value in quotes"));
        assert_eq!(env.get("SINGLE"), Some("single quotes"));
        assert_eq!(env.get("NONE"), Some("no quotes"));
    }

    #[test]
    fn test_env_save_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".env");

        let pairs = vec![
            ("KEY1".to_string(), "value1".to_string()),
            ("KEY2".to_string(), "value with space".to_string()),
        ];
        let env = Env::from_pairs(pairs, path.clone());

        env.save().unwrap();

        let loaded = Env::load(&path).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.get("KEY1"), Some("value1"));
        assert_eq!(loaded.get("KEY2"), Some("value with space"));
    }

    #[test]
    fn test_env_empty() {
        let env = Env::from_pairs(vec![], PathBuf::from(".env"));

        assert!(env.is_empty());
        assert_eq!(env.len(), 0);
        assert_eq!(env.entries().len(), 0);
    }
}
