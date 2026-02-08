//! Sync command - re-encrypt secrets for the current recipient set.

use tracing::info;

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Sync secrets for the current recipient set.
pub fn execute(dry_run: bool, force: bool) -> Result<()> {
    info!(dry_run, force, "running sync");

    let mut vault = Vault::open()?;

    if dry_run {
        if vault.needs_sync() || force {
            let secrets = vault.config().secrets.len();
            let recipients = vault.config().recipients.len();
            output::warn(&format!(
                "would sync ({} secrets, {} recipients)",
                secrets, recipients
            ));
        } else {
            output::success("already in sync");
        }
        return Ok(());
    }

    let result = vault.sync(force)?;

    if result.was_needed {
        output::success(&format!(
            "synced ({} secrets, {} recipients)",
            result.secrets, result.recipients
        ));
    } else {
        output::success("already in sync");
    }

    Ok(())
}
