//! Status command - show quick status overview.

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Show quick status overview.
pub fn execute(vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;
    let v = Vault::open_vault(vault_name.as_deref())?;

    // Project name
    let vault_display = vault_name
        .as_ref()
        .map(|n| format!(".dugout.{}.toml", n))
        .unwrap_or_else(|| ".dugout.toml".to_string());
    output::kv("vault", vault_display);

    // Cipher backend
    let backend_name = if v.config().has_kms() {
        "hybrid (age + kms)"
    } else {
        "age"
    };
    output::kv("cipher", backend_name);

    // Secret count
    let secret_count = v.list().len();
    output::kv("secrets", secret_count);

    // Team member count
    let team_count = v.recipients().len();
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
