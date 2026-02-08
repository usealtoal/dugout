//! Team add command - add a team member.

use tracing::info;

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Add a team member.
pub fn execute(name: &str, key: &str) -> Result<()> {
    info!("Adding team member: {}", name);
    let mut vault = Vault::open()?;
    vault.add_recipient(name, key)?;
    output::success(&format!("added {}", name));
    Ok(())
}
