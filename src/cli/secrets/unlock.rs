//! Unlock command.
//!
//! Decrypts secrets to local .env file.

use crate::cli::output;
use crate::error::Result;

/// Unlock secrets to .env file.
pub fn execute() -> Result<()> {
    let vault = crate::core::vault::Vault::open()?;
    output::progress("Decrypting secrets");
    let env = vault.unlock()?;
    output::progress_done(true);
    output::success(&format!(
        "unlocked: {} secrets written to {}",
        env.len(),
        output::path(".env")
    ));
    Ok(())
}
