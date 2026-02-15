//! Lock command - verify encryption status.

use crate::cli::output;
use crate::error::Result;

/// Lock (status check - secrets are always encrypted).
pub fn execute(vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;
    let v = crate::core::vault::Vault::open_vault(vault_name.as_deref())?;
    let count = v.list().len();
    output::success(&format!("locked ({} secrets)", count));
    Ok(())
}
