//! Unlock command - decrypt secrets to .env file.

use crate::cli::output;
use crate::error::Result;

/// Unlock secrets to .env file.
pub fn execute() -> Result<()> {
    let vault = crate::core::vault::Vault::open()?;
    let env = vault.unlock()?;
    output::success(&format!("unlocked to .env ({} secrets)", env.len()));
    Ok(())
}
