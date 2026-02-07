//! Import command.
//!
//! Import secrets from a .env file.

use crate::cli::output;
use crate::error::Result;

/// Import secrets from a .env file.
pub fn execute(path: &str) -> Result<()> {
    let mut vault = crate::core::vault::Vault::open()?;
    let imported = vault.import(path)?;
    output::success(&format!(
        "imported {} secrets from {}",
        imported.len(),
        output::path(path)
    ));
    for key in &imported {
        output::list_item(key);
    }
    Ok(())
}
