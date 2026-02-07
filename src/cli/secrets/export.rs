//! Export command.
//!
//! Export secrets as .env format to stdout.

use crate::cli::output;
use crate::error::Result;

/// Export secrets as .env format to stdout.
pub fn execute() -> Result<()> {
    let vault = crate::core::vault::Vault::open()?;
    let env = vault.export()?;
    output::raw(&env.to_string());
    Ok(())
}
