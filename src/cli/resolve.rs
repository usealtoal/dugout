//! Vault resolution helpers for CLI commands.

use crate::core::vault::Vault;
use crate::error::{ConfigError, Result, ValidationError};

/// Validate a vault name.
///
/// Vault names must be safe for use in file paths:
/// - Not empty
/// - Not "." or ".."
/// - No path separators (/ or \)
/// - Only alphanumeric, underscore, hyphen, and dot
/// - Max 64 characters
pub fn validate_vault_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(ValidationError::InvalidVaultName {
            name: name.to_string(),
            reason: "vault name cannot be empty".to_string(),
        }
        .into());
    }

    if name == "." || name == ".." {
        return Err(ValidationError::InvalidVaultName {
            name: name.to_string(),
            reason: "vault name cannot be '.' or '..'".to_string(),
        }
        .into());
    }

    if name.len() > 64 {
        return Err(ValidationError::InvalidVaultName {
            name: name.to_string(),
            reason: "vault name must be at most 64 characters".to_string(),
        }
        .into());
    }

    for (i, ch) in name.chars().enumerate() {
        if ch == '/' || ch == '\\' {
            return Err(ValidationError::InvalidVaultName {
                name: name.to_string(),
                reason: format!(
                    "invalid character '{}' at position {}. Path separators are not allowed",
                    ch,
                    i + 1
                ),
            }
            .into());
        }
        if !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-' && ch != '.' {
            return Err(ValidationError::InvalidVaultName {
                name: name.to_string(),
                reason: format!(
                    "invalid character '{}' at position {}. Allowed: A-Z, a-z, 0-9, _, -, .",
                    ch,
                    i + 1
                ),
            }
            .into());
        }
    }

    Ok(())
}

/// Resolve which vault to use based on CLI flag and available vaults.
///
/// Rules:
/// - If vault is specified, validate and use it
/// - If only one vault exists, use it
/// - If multiple vaults exist, error with guidance
pub fn resolve_vault(vault: Option<&str>) -> Result<Option<String>> {
    // If explicit vault specified, validate and use it
    if let Some(v) = vault {
        validate_vault_name(v)?;
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
pub fn resolve_vault_default(vault: Option<&str>) -> Result<Option<String>> {
    if let Some(v) = vault {
        validate_vault_name(v)?;
        return Ok(Some(v.to_string()));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_vault_name_valid() {
        assert!(validate_vault_name("dev").is_ok());
        assert!(validate_vault_name("prod").is_ok());
        assert!(validate_vault_name("staging-1").is_ok());
        assert!(validate_vault_name("test_env").is_ok());
        assert!(validate_vault_name("v1.0").is_ok());
    }

    #[test]
    fn test_validate_vault_name_rejects_empty() {
        assert!(validate_vault_name("").is_err());
    }

    #[test]
    fn test_validate_vault_name_rejects_dots() {
        assert!(validate_vault_name(".").is_err());
        assert!(validate_vault_name("..").is_err());
    }

    #[test]
    fn test_validate_vault_name_rejects_path_separators() {
        assert!(validate_vault_name("foo/bar").is_err());
        assert!(validate_vault_name("foo\\bar").is_err());
        assert!(validate_vault_name("../secrets").is_err());
    }

    #[test]
    fn test_validate_vault_name_rejects_special_chars() {
        assert!(validate_vault_name("foo bar").is_err());
        assert!(validate_vault_name("foo@bar").is_err());
        assert!(validate_vault_name("foo:bar").is_err());
    }
}
