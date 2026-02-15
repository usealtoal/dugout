//! Import command - import secrets from a .env file.

use crate::cli::output;
use crate::error::Result;

/// Import secrets from a .env file.
pub fn execute(path: &str, vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;
    let mut v = crate::core::vault::Vault::open_vault(vault_name.as_deref())?;
    let imported = v.import(path)?;
    output::success(&format!(
        "imported {} secrets from {}",
        imported.len(),
        path
    ));
    Ok(())
}
