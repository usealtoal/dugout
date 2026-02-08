//! Diff command.
//!
//! Show diff between .burrow.toml and .env.

use crate::cli::output;
use crate::error::Result;

/// Show diff/status between encrypted and local .env.
pub fn execute() -> Result<()> {
    let vault = crate::core::vault::Vault::open()?;
    let env_path = std::path::Path::new(".env");
    let diff = vault.diff(env_path)?;

    output::section("Diff");

    // Synced entries
    let synced = diff.synced();
    if !synced.is_empty() {
        output::success("synced:");
        for entry in &synced {
            output::list_item(&output::key(entry.key()));
        }
        output::blank();
    }

    // Modified entries
    let modified = diff.modified();
    if !modified.is_empty() {
        output::warn("modified (values differ):");
        for entry in &modified {
            output::list_item(&output::key(entry.key()));
        }
        output::blank();
        output::hint(&format!(
            "run {} to update .env with vault values",
            output::cmd("burrow secrets unlock")
        ));
    }

    // Vault-only entries
    let vault_only = diff.vault_only();
    if !vault_only.is_empty() {
        output::warn("vault only:");
        for entry in &vault_only {
            output::list_item(&output::key(entry.key()));
        }
        output::blank();
        output::hint(&format!(
            "run {} to sync these secrets",
            output::cmd("burrow secrets unlock")
        ));
    }

    // Env-only entries
    let env_only = diff.env_only();
    if !env_only.is_empty() {
        output::warn("env only:");
        for entry in &env_only {
            output::list_item(&output::key(entry.key()));
        }
        output::blank();
        output::hint(&format!(
            "run {} to encrypt untracked secrets",
            output::cmd("burrow secrets import .env")
        ));
    }

    // Summary
    if diff.is_empty() {
        if env_path.exists() {
            output::warn(".env is empty");
        } else {
            output::warn(".env file not found");
        }
        output::blank();
        output::hint(&format!(
            "run {} to create .env file",
            output::cmd("burrow secrets unlock")
        ));
    } else if diff.is_synced() {
        output::success("all synced");
    }

    Ok(())
}
