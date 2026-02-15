//! Team add command - add a team member.

use tracing::info;

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Add a team member.
pub fn execute(name: &str, key: &str, vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;
    info!("Adding team member: {}", name);
    let mut v = Vault::open_vault(vault_name.as_deref())?;
    v.add_recipient(name, key)?;
    output::success(&format!("added {}", name));
    Ok(())
}
