//! Vault information for listing vaults.

use std::path::PathBuf;

/// Information about a vault for listing.
#[derive(Debug, Clone)]
pub struct VaultInfo {
    /// Vault name ("default", "dev", "prod")
    pub name: String,
    /// Path to vault file
    pub path: PathBuf,
    /// Number of secrets
    pub secret_count: usize,
    /// Number of recipients
    pub recipient_count: usize,
    /// Whether current identity has access
    pub has_access: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_info_creation() {
        let info = VaultInfo {
            name: "dev".to_string(),
            path: PathBuf::from(".dugout.dev.toml"),
            secret_count: 5,
            recipient_count: 2,
            has_access: true,
        };
        assert_eq!(info.name, "dev");
        assert_eq!(info.path, PathBuf::from(".dugout.dev.toml"));
        assert_eq!(info.secret_count, 5);
        assert_eq!(info.recipient_count, 2);
        assert!(info.has_access);
    }
}
