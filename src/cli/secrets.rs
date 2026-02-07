//! Secret management commands.
//!
//! Implements set, get, rm, and list operations for secrets.

use tracing::info;

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Set a secret value.
pub fn set(key: &str, value: &str, force: bool) -> Result<()> {
    info!("Setting secret: {} (force: {})", key, force);
    let mut vault = Vault::open()?;
    vault.set(key, value, force)?;
    output::success(&format!("set: {}", output::key(key)));
    Ok(())
}

/// Get a secret value.
pub fn get(key: &str) -> Result<()> {
    let vault = Vault::open()?;
    let value = vault.get(key)?;
    // Plain output for scripting - no decoration
    println!("{}", value.as_str());
    Ok(())
}

/// Remove a secret.
pub fn rm(key: &str) -> Result<()> {
    info!("Removing secret: {}", key);
    let mut vault = Vault::open()?;
    vault.remove(key)?;
    output::success(&format!("removed: {}", output::key(key)));
    Ok(())
}

/// List all secret keys.
pub fn list(json: bool) -> Result<()> {
    let vault = Vault::open()?;
    let secrets = vault.list();

    if json {
        let keys: Vec<String> = secrets.iter().map(|s| s.key().to_string()).collect();
        let result = serde_json::json!({
            "keys": keys,
            "count": secrets.len()
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if secrets.is_empty() {
        output::dimmed("no secrets stored");
    } else {
        println!();
        output::header(&format!("{} secrets", secrets.len()));
        output::rule();
        for secret in secrets {
            output::list_item(secret.key());
        }
    }

    Ok(())
}
