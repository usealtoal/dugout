//! Team add command.
//!
//! Add a team member by their public key.

use tracing::info;

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Add a team member.
pub fn execute(name: &str, key: &str) -> Result<()> {
    info!("Adding team member: {}", name);
    let mut vault = Vault::open()?;
    let secret_count = vault.list().len();

    let sp = output::spinner(&format!("Adding member {}...", output::key(name)));
    vault.add_recipient(name, key)?;
    output::spinner_success(&sp, &format!("Added team member {}", output::key(name)));

    if secret_count > 0 {
        output::dimmed(&format!(
            "  re-encrypted {} secrets for new recipient",
            secret_count
        ));
    }
    Ok(())
}
