//! Team remove command - remove a team member.

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Remove a team member.
pub fn execute(name: &str) -> Result<()> {
    let mut vault = Vault::open()?;
    vault.remove_recipient(name)?;
    output::success(&format!("removed {}", name));
    Ok(())
}
