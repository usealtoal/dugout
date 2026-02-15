//! Team remove command - remove a team member.

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Remove a team member.
pub fn execute(name: &str, vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;
    let mut v = Vault::open_vault(vault_name.as_deref())?;
    v.remove_recipient(name)?;
    output::success(&format!("removed {}", name));
    Ok(())
}
