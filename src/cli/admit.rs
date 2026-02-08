//! Admit command - approve an access request.

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Admit a team member from a pending request.
pub fn execute(name: &str) -> Result<()> {
    let mut vault = Vault::open()?;
    vault.admit(name)?;
    output::success(&format!("admitted {}", name));
    Ok(())
}
