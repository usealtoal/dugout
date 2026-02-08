//! Lock command - verify encryption status.

use crate::cli::output;
use crate::error::Result;

/// Lock (status check - secrets are always encrypted).
pub fn execute() -> Result<()> {
    let vault = crate::core::vault::Vault::open()?;
    let count = vault.list().len();
    output::success(&format!("locked ({} secrets)", count));
    Ok(())
}
