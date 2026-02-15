//! Constants used throughout dugout.
//!
//! Centralizes magic strings and configuration values.

/// Configuration file name (.dugout.toml).
pub const CONFIG_FILE: &str = ".dugout.toml";

/// Environment variables file name (.env).
#[allow(dead_code)]
pub const ENV_FILE: &str = ".env";

/// Key storage directory relative to HOME (~/.dugout/keys).
pub const KEY_DIR: &str = ".dugout/keys";

/// Gitignore entries to protect secrets.
///
/// These entries ensure that .env files are not accidentally committed.
pub const GITIGNORE_ENTRIES: &[&str] = &[".env", ".env.*", "!.env.example"];

/// Check if a vault name is safe for path construction.
///
/// Defense in depth: rejects path separators and special names even if CLI already validated.
fn is_safe_vault_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
}

/// Get vault file path for given vault name.
///
/// - `None` → `.dugout.toml` (default)
/// - `Some("default")` → `.dugout.toml` (default is alias for None)
/// - `Some("dev")` → `.dugout.dev.toml`
///
/// # Panics
///
/// Panics if vault name contains path separators (defense in depth).
pub fn vault_path(vault: Option<&str>) -> std::path::PathBuf {
    match vault {
        None | Some("default") => std::path::PathBuf::from(CONFIG_FILE),
        Some(name) => {
            assert!(is_safe_vault_name(name), "unsafe vault name: {}", name);
            std::path::PathBuf::from(format!(".dugout.{}.toml", name))
        }
    }
}

/// Get request directory for given vault.
///
/// - `None` → `.dugout/requests/default`
/// - `Some("default")` → `.dugout/requests/default` (alias for None)
/// - `Some("prod")` → `.dugout/requests/prod`
///
/// # Panics
///
/// Panics if vault name contains path separators (defense in depth).
pub fn request_dir(vault: Option<&str>) -> std::path::PathBuf {
    let base = std::path::PathBuf::from(".dugout/requests");
    match vault {
        None | Some("default") => base.join("default"),
        Some(name) => {
            assert!(is_safe_vault_name(name), "unsafe vault name: {}", name);
            base.join(name)
        }
    }
}

/// Extract vault name from a vault file path.
///
/// - `.dugout.toml` → `None` (default)
/// - `.dugout.dev.toml` → `Some("dev")`
pub fn vault_name_from_path(path: &std::path::Path) -> Option<String> {
    let filename = path.file_name()?.to_str()?;
    if filename == CONFIG_FILE {
        return None;
    }
    // Pattern: .dugout.{name}.toml
    if filename.starts_with(".dugout.") && filename.ends_with(".toml") {
        let name = filename.strip_prefix(".dugout.")?.strip_suffix(".toml")?;
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_path_default() {
        assert_eq!(vault_path(None), std::path::PathBuf::from(".dugout.toml"));
    }

    #[test]
    fn test_vault_path_named() {
        assert_eq!(
            vault_path(Some("dev")),
            std::path::PathBuf::from(".dugout.dev.toml")
        );
        assert_eq!(
            vault_path(Some("prod")),
            std::path::PathBuf::from(".dugout.prod.toml")
        );
    }

    #[test]
    fn test_request_dir_default() {
        assert_eq!(
            request_dir(None),
            std::path::PathBuf::from(".dugout/requests/default")
        );
    }

    #[test]
    fn test_request_dir_named() {
        assert_eq!(
            request_dir(Some("prod")),
            std::path::PathBuf::from(".dugout/requests/prod")
        );
    }

    #[test]
    fn test_vault_name_from_path_default() {
        let path = std::path::Path::new(".dugout.toml");
        assert_eq!(vault_name_from_path(path), None);
    }

    #[test]
    fn test_vault_name_from_path_named() {
        let path = std::path::Path::new(".dugout.dev.toml");
        assert_eq!(vault_name_from_path(path), Some("dev".to_string()));
    }

    #[test]
    fn test_vault_name_from_path_empty_name() {
        // .dugout..toml should return None, not Some("")
        let path = std::path::Path::new(".dugout..toml");
        assert_eq!(vault_name_from_path(path), None);
    }

    #[test]
    fn test_vault_name_from_path_invalid() {
        // Non-vault files should return None
        assert_eq!(vault_name_from_path(std::path::Path::new("config.toml")), None);
        assert_eq!(vault_name_from_path(std::path::Path::new(".env")), None);
        assert_eq!(vault_name_from_path(std::path::Path::new("dugout.toml")), None);
    }
}
