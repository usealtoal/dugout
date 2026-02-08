//! Pending command.
//!
//! List pending access requests.

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// List pending access requests.
pub fn execute() -> Result<()> {
    let vault = Vault::open()?;
    let requests = vault.pending_requests()?;

    if requests.is_empty() {
        output::blank();
        output::dimmed("no pending requests");
        return Ok(());
    }

    output::blank();
    output::header(&format!(
        "{} pending requests",
        output::count(requests.len())
    ));
    output::rule();

    for (name, pubkey) in requests {
        let truncated = if pubkey.len() > 50 {
            format!("{}...", &pubkey[..50])
        } else {
            pubkey
        };
        output::kv(&name, truncated);
    }

    output::blank();
    output::hint(&format!(
        "run {} to approve a request",
        output::cmd("burrow admit <name>")
    ));

    Ok(())
}
