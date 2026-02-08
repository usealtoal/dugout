//! Diff command - show differences between vault and .env.

use crate::cli::output;
use crate::error::Result;

/// Show diff/status between encrypted vault and local .env.
pub fn execute() -> Result<()> {
    let vault = crate::core::vault::Vault::open()?;
    let env_path = std::path::Path::new(".env");

    if !env_path.exists() {
        output::warn(".env not found");
        return Ok(());
    }

    let diff = vault.diff(env_path)?;

    // Vault-only entries
    let vault_only = diff.vault_only();
    for entry in &vault_only {
        println!("+ {} (vault only)", entry.key());
    }

    // Env-only entries
    let env_only = diff.env_only();
    for entry in &env_only {
        println!("- {} (env only)", entry.key());
    }

    // Modified entries
    let modified = diff.modified();
    for entry in &modified {
        println!("~ {} (modified)", entry.key());
    }

    // Synced entries
    let synced = diff.synced();
    for entry in &synced {
        println!("✓ {}", entry.key());
    }

    Ok(())
}
