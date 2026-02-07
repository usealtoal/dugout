//! Secret management commands (set, get, rm, list).

use colored::Colorize;

use crate::core::config::Config;
use crate::core::secrets;
use crate::error::Result;

/// Set a secret value.
pub fn set(key: &str, value: &str, force: bool) -> Result<()> {
    let mut config = Config::load()?;
    secrets::set(&mut config, key, value, force)?;
    println!("{} {}", "set:".green().bold(), key);
    Ok(())
}

/// Get a secret value.
pub fn get(key: &str) -> Result<()> {
    let config = Config::load()?;
    let value = secrets::get(&config, key)?;
    println!("{}", value.as_str());
    Ok(())
}

/// Remove a secret.
pub fn rm(key: &str) -> Result<()> {
    let mut config = Config::load()?;
    secrets::remove(&mut config, key)?;
    println!("{} {}", "removed:".green().bold(), key);
    Ok(())
}

/// List all secret keys.
pub fn list(json: bool) -> Result<()> {
    let config = Config::load()?;
    let keys = secrets::list(&config);

    if json {
        let output = serde_json::json!({
            "keys": keys,
            "count": keys.len()
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else if keys.is_empty() {
        println!("{}", "no secrets stored".dimmed());
    } else {
        println!("{} secrets:", keys.len().to_string().green().bold());
        for key in keys {
            println!("  {}", key);
        }
    }

    Ok(())
}
