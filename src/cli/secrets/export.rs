//! Export command - export secrets as .env format to stdout.

use crate::cli::output;
use crate::error::Result;

/// Export secrets as .env format to stdout.
pub fn execute(vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;
    let v = crate::core::vault::Vault::open_vault(vault_name.as_deref())?;
    let env = v.export()?;
    output::raw(&env.to_string());
    Ok(())
}
