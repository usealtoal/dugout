//! Sync command - re-encrypt secrets for the current recipient set.

use tracing::info;

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Sync secrets for the current recipient set.
pub fn execute(dry_run: bool, force: bool, vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;
    info!(dry_run, force, "running sync");

    let mut v = Vault::open_vault(vault_name.as_deref())?;

    if dry_run {
        if v.needs_sync() || force {
            let secrets = v.config().secrets.len();
            let recipients = v.config().recipients.len();
            output::warn(&format!(
                "would sync ({} secrets, {} recipients)",
                secrets, recipients
            ));
        } else {
            output::success("already in sync");
        }
        return Ok(());
    }

    let result = v.sync(force)?;

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
