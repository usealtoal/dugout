//! Status command - show quick status overview.

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Show quick status overview.
pub fn execute() -> Result<()> {
    let vault = Vault::open()?;

    // Project name
    output::kv("vault", ".burrow.toml");

    // Cipher backend
    output::kv("cipher", "age");

    // Secret count
    let secret_count = vault.list().len();
    output::kv("secrets", secret_count);

    // Team member count
    let team_count = vault.recipients().len();
    let team_label = if team_count == 1 {
        "1 member"
    } else {
        "members"
    };
    if team_count == 1 {
        output::kv("team", team_label);
    } else {
        output::kv("team", format!("{} {}", team_count, team_label));
    }

    Ok(())
}
