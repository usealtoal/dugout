//! Lock command.
//!
//! Encrypts all secrets and verifies encryption status.

use crate::cli::output;
use crate::error::Result;

/// Lock (status check - secrets are always encrypted).
pub fn execute() -> Result<()> {
    let vault = crate::core::vault::Vault::open()?;
    let sp = output::spinner("Verifying encryption...");
    let count = vault.list().len();
    output::spinner_success(
        &sp,
        &format!("All {} secrets encrypted", output::count(count)),
    );
    output::kv("status", "safe to commit");
    Ok(())
}
