//! Lock and unlock commands.

use colored::Colorize;

use crate::core::config::Config;
use crate::core::env;
use crate::error::Result;

/// Lock (status check - secrets are always encrypted).
pub fn lock() -> Result<()> {
    let config = Config::load()?;
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
    let config = Config::load()?;
    let count = env::unlock(&config)?;
    println!(
        "{} {} secrets written to .env",
        "unlocked:".green().bold(),
        count
    );
    Ok(())
}
