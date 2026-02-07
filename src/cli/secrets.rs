//! Secret management commands.
//!
//! Implements set, get, rm, and list operations for secrets.

use crate::cli::output;
use crate::core::config::Config;
use crate::core::secrets;
use crate::error::Result;

/// Set a secret value.
pub fn set(key: &str, value: &str, force: bool) -> Result<()> {
    let mut config = Config::load()?;
    secrets::set(&mut config, key, value, force)?;
    output::success(&format!("set: {}", output::key(key)));
    Ok(())
}

/// Get a secret value.
pub fn get(key: &str) -> Result<()> {
    let config = Config::load()?;
    let value = secrets::get(&config, key)?;
    // Plain output for scripting - no decoration
    println!("{}", value.as_str());
    Ok(())
}

/// Remove a secret.
pub fn rm(key: &str) -> Result<()> {
    let mut config = Config::load()?;
    secrets::remove(&mut config, key)?;
    output::success(&format!("removed: {}", output::key(key)));
    Ok(())
}

/// List all secret keys.
pub fn list(json: bool) -> Result<()> {
    let config = Config::load()?;
    let keys = secrets::list(&config);

    if json {
        let result = serde_json::json!({
            "keys": keys,
            "count": keys.len()
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if keys.is_empty() {
        output::dimmed("no secrets stored");
    } else {
        println!();
        output::header(&format!("{} secrets", keys.len()));
        output::rule();
        for key in keys {
            output::list_item(&key);
        }
    }

    Ok(())
}
