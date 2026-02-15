//! Unlock command - decrypt secrets to .env file.

use crate::cli::output;
use crate::error::Result;

/// Unlock secrets to .env file.
pub fn execute(vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;
    let v = crate::core::vault::Vault::open_vault(vault_name.as_deref())?;
    let env = v.unlock()?;
    output::success(&format!("unlocked to .env ({} secrets)", env.len()));
    Ok(())
}
