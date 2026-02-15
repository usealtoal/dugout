//! Vault resolution helpers for CLI commands.

use crate::core::vault::Vault;
use crate::error::{ConfigError, Result};

/// Resolve which vault to use based on CLI flag and available vaults.
///
/// Rules:
/// - If vault is specified, use it
/// - If only one vault exists, use it
/// - If multiple vaults exist, error with guidance
pub fn resolve_vault(vault: Option<&str>) -> Result<Option<String>> {
    // If explicit vault specified, use it
    if let Some(v) = vault {
        return Ok(Some(v.to_string()));
    }

    // Check how many vaults exist
    let vault_files = Vault::find_vault_files()?;

    match vault_files.len() {
        0 => Ok(None), // No vaults - let command handle NotInitialized
        1 => {
            // Single vault - extract name from path
            let path = &vault_files[0];
            let name = crate::core::constants::vault_name_from_path(path);
            Ok(name)
        }
        _ => {
            // Multiple vaults - require explicit selection
            let vaults_list: Vec<String> = vault_files
                .iter()
                .map(|p| {
                    let name = crate::core::constants::vault_name_from_path(p);
                    match name {
                        Some(n) => format!("  .dugout.{}.toml ({})", n, n),
                        None => "  .dugout.toml (default)".to_string(),
                    }
                })
                .collect();

            Err(ConfigError::MultipleVaults {
                vaults: vaults_list.join("\n"),
            }
            .into())
        }
    }
}

/// Resolve vault for commands that default to .dugout.toml (like `dugout .`).
///
/// Unlike resolve_vault(), this always defaults to None (the default vault)
/// if no explicit vault is specified, regardless of how many vaults exist.
pub fn resolve_vault_default(vault: Option<&str>) -> Option<String> {
    vault.map(|s| s.to_string())
}
