//! Secret management commands (set, get, rm, list).

use colored::Colorize;

use crate::core::config::BurrowConfig;
use crate::core::secrets;
use crate::error::Result;

/// Set a secret value.
pub fn set(key: &str, value: &str, force: bool) -> Result<()> {
    let mut config = BurrowConfig::load()?;
    secrets::set_secret(&mut config, key, value, force)?;
    println!("{} {}", "set:".green().bold(), key);
    Ok(())
}

/// Get a secret value.
pub fn get(key: &str) -> Result<()> {
    let config = BurrowConfig::load()?;
    let value = secrets::get_secret(&config, key)?;
    println!("{}", value);
    Ok(())
}

/// Remove a secret.
pub fn rm(key: &str) -> Result<()> {
    let mut config = BurrowConfig::load()?;
    secrets::remove_secret(&mut config, key)?;
    println!("{} {}", "removed:".green().bold(), key);
    Ok(())
}

/// List all secret keys.
pub fn list() -> Result<()> {
    let config = BurrowConfig::load()?;
    let keys = secrets::list_secrets(&config);

    if keys.is_empty() {
        println!("{}", "no secrets stored".dimmed());
    } else {
        println!("{} secrets:", keys.len().to_string().green().bold());
        for key in keys {
            println!("  {}", key);
        }
    }

    Ok(())
}
