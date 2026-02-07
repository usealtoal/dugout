//! Import and export operations for .env files.
//!
//! Provides seamless integration with dotenv-style environment files.

use crate::core::config::Config;
use crate::core::secrets;
use crate::error::Result;

/// Import secrets from a .env file.
///
/// Parses a .env file and encrypts each key-value pair into the burrow.
/// Skips empty lines and comments.
///
/// # Arguments
///
/// * `config` - Mutable reference to configuration
/// * `path` - Path to the .env file
///
/// # Returns
///
/// Vector of imported key names.
///
/// # Errors
///
/// Returns error if file reading or encryption fails.
pub fn import(config: &mut Config, path: &str) -> Result<Vec<String>> {
    let contents = std::fs::read_to_string(path)?;
    let mut imported = Vec::new();

    for line in contents.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');

            secrets::set(config, key, value, true)?;
            imported.push(key.to_string());
        }
    }

    Ok(imported)
}

/// Export secrets as .env format string.
///
/// Decrypts all secrets and formats them as KEY=value pairs.
/// Quotes values containing spaces or special characters.
///
/// # Arguments
///
/// * `config` - Configuration reference
///
/// # Returns
///
/// .env-formatted string of all secrets.
///
/// # Errors
///
/// Returns error if decryption fails.
pub fn export(config: &Config) -> Result<String> {
    let pairs = secrets::decrypt_all(config)?;
    let mut output = String::new();

    for (key, value) in pairs {
        // Quote values that contain spaces or special chars
        if value.contains(' ') || value.contains('#') || value.contains('=') {
            output.push_str(&format!("{}=\"{}\"\n", key, value));
        } else {
            output.push_str(&format!("{}={}\n", key, value));
        }
    }

    Ok(output)
}

/// Write decrypted secrets to a .env file.
///
/// Unlocks all secrets and writes them to `.env` in the current directory.
///
/// # Arguments
///
/// * `config` - Configuration reference
///
/// # Returns
///
/// Number of secrets written.
///
/// # Errors
///
/// Returns error if decryption or file write fails.
pub fn unlock(config: &Config) -> Result<usize> {
    let env_content = export(config)?;
    let count = env_content.lines().count();

    std::fs::write(".env", env_content)?;

    Ok(count)
}
