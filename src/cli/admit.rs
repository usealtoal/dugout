//! Admit command - approve an access request.

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Admit a team member from a pending request.
pub fn execute(name: &str, vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;
    let mut v = Vault::open_vault(vault_name.as_deref())?;
    v.admit(name)?;
    output::success(&format!("admitted {}", name));
    Ok(())
}
