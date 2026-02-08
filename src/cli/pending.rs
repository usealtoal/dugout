//! Pending command - list pending access requests.

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// List pending access requests.
pub fn execute() -> Result<()> {
    let vault = Vault::open()?;
    let requests = vault.pending_requests()?;

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
