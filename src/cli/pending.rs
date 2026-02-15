//! Pending command - list pending access requests.

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// List pending access requests.
pub fn execute(vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;
    let v = Vault::open_vault(vault_name.as_deref())?;
    let requests = v.pending_requests()?;

    if requests.is_empty() {
        output::data("no pending requests");
        return Ok(());
    }

    for (name, pubkey) in requests {
        let truncated = if pubkey.len() > 20 {
            format!("{}...", &pubkey[..20])
        } else {
            pubkey
        };
        println!("{:<15} {}", name, truncated);
    }

    Ok(())
}
