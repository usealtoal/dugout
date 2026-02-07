//! Team remove command.
//!
//! Remove a team member.

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Remove a team member.
pub fn execute(name: &str) -> Result<()> {
    let sp = output::spinner(&format!("Removing member {}...", output::key(name)));
    let mut vault = Vault::open()?;
    vault.remove_recipient(name)?;
    output::spinner_success(&sp, &format!("Removed team member {}", output::key(name)));
    Ok(())
}
