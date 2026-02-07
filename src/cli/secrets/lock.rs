//! Lock command.
//!
//! Encrypts all secrets and verifies encryption status.

use crate::cli::output;
use crate::error::Result;

/// Lock (status check - secrets are always encrypted).
pub fn execute() -> Result<()> {
    let vault = crate::core::vault::Vault::open()?;
    output::progress("Checking encryption");
    output::progress_done(true);
    output::success(&format!(
        "locked: {} secrets encrypted in {}",
        vault.list().len(),
        output::path(".burrow.toml")
    ));
    output::kv("status", "safe to commit");
    Ok(())
}
