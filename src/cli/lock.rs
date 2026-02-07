//! Lock and unlock commands.

use colored::Colorize;

use crate::core::config::BurrowConfig;
use crate::core::env;
use crate::error::Result;

/// Lock (status check - secrets are always encrypted).
pub fn lock() -> Result<()> {
    let config = BurrowConfig::load()?;
    println!(
        "{} {} secrets encrypted in .burrow.toml",
        "locked:".green().bold(),
        config.secrets.len()
    );
    println!("  safe to commit");
    Ok(())
}

/// Unlock secrets to .env file.
pub fn unlock() -> Result<()> {
    let config = BurrowConfig::load()?;
    let count = env::unlock_to_file(&config)?;
    println!(
        "{} {} secrets written to .env",
        "unlocked:".green().bold(),
        count
    );
    Ok(())
}
