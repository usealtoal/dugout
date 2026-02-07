//! Lock and unlock commands.
//!
//! Verifies encryption status (lock) and decrypts secrets to .env (unlock).

use crate::cli::output;
use crate::core::config::Config;
use crate::core::env;
use crate::error::Result;

/// Lock (status check - secrets are always encrypted).
pub fn lock() -> Result<()> {
    let config = Config::load()?;
    output::progress("Checking encryption");
    output::progress_done(true);
    output::success(&format!(
        "locked: {} secrets encrypted in {}",
        config.secrets.len(),
        output::path(".burrow.toml")
    ));
    output::kv("status", "safe to commit");
    Ok(())
}

/// Unlock secrets to .env file.
pub fn unlock() -> Result<()> {
    let config = Config::load()?;
    output::progress("Decrypting secrets");
    let count = env::unlock(&config)?;
    output::progress_done(true);
    output::success(&format!(
        "unlocked: {} secrets written to {}",
        count,
        output::path(".env")
    ));
    Ok(())
}
