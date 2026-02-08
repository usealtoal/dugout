//! Admit command.
//!
//! Approve an access request and add the user to the team.

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Admit a team member from a pending request.
pub fn execute(name: &str) -> Result<()> {
    let sp = output::spinner(&format!("admitting {}...", name));
    let mut vault = Vault::open()?;

    // This will read the request file, add the recipient, and delete the request
    vault.admit(name)?;

    output::spinner_success(&sp, &format!("admitted {}", output::key(name)));

    output::blank();
    output::success(&format!("{} now has access to this vault", name));
    output::blank();
    output::hint("all secrets have been re-encrypted for the new team");

    Ok(())
}
