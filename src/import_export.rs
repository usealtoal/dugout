use std::path::Path;

use crate::config::BurrowConfig;
use crate::error::Result;
use crate::secrets;

/// Import secrets from a .env file
pub fn import_env(config: &mut BurrowConfig, path: &str) -> Result<Vec<String>> {
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

            secrets::set_secret(config, key, value, true)?;
            imported.push(key.to_string());
        }
    }

    Ok(imported)
}

/// Export secrets as .env format string
pub fn export_env(config: &BurrowConfig) -> Result<String> {
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

/// Write decrypted secrets to a .env file
pub fn unlock_to_file(config: &BurrowConfig) -> Result<usize> {
    let env_content = export_env(config)?;
    let count = env_content.lines().count();

    std::fs::write(".env", env_content)?;

    Ok(count)
}
