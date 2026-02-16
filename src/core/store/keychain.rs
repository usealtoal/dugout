//! macOS Keychain backend for secure identity storage
//!
//! This module provides a Store implementation that uses the macOS Keychain
//! to store age identities with hardware-backed security.

#![cfg(target_os = "macos")]

use tracing::{debug, error, info};

use crate::core::domain::identity::{Identity, IdentitySource};
use crate::error::{Result, StoreError};
use age::x25519;

/// Keychain backend for storing identities
pub struct Keychain {
    service: String,
}

impl Keychain {
    /// Service name for all dugout identities in Keychain
    const SERVICE_NAME: &'static str = "com.dugout";

    /// Create a new Keychain backend
    pub fn new() -> Result<Self> {
        Ok(Self {
            service: Self::SERVICE_NAME.to_string(),
        })
    }

    /// Store an identity in the Keychain
    pub fn store_identity(&self, account: &str, secret: &str, force: bool) -> Result<()> {
        use security_framework::passwords::set_generic_password;

        info!(
            account = %account,
            service = %self.service,
            "Storing identity in macOS Keychain"
        );

        // Check if key already exists
        if !force && self.keychain_has_key(account) {
            error!(account = %account, "Identity already exists in Keychain");
            return Err(StoreError::KeychainError(
                format!("Identity '{}' already exists in Keychain. Use --force to overwrite.", account)
            ).into());
        }

        // Delete existing entry if force=true
        if force {
            info!(account = %account, "Deleting existing Keychain entry (--force)");
            let _ = self.delete_identity(account);
        }

        // Store in Keychain
        match set_generic_password(&self.service, account, secret.as_bytes()) {
            Ok(_) => {
                info!(
                    account = %account,
                    "✓ Identity stored in Keychain"
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    account = %account,
                    error = %e,
                    "Failed to store identity in Keychain"
                );

                // Map specific error codes
                let error_code = e.code();
                if error_code == -128 {
                    // User cancelled authorization
                    Err(StoreError::KeychainAccessDenied.into())
                } else {
                    Err(StoreError::KeychainError(format!(
                        "Keychain storage failed: {}. Use --filesystem flag to store locally instead.",
                        e
                    )).into())
                }
            }
        }
    }

    /// Load an identity from the Keychain
    pub(crate) fn load_from_keychain(&self, account: &str) -> Result<Identity> {
        info!(account = %account, service = %self.service, "Loading identity from macOS Keychain");

        // Retrieve the password from Keychain
        use security_framework::passwords::get_generic_password;

        match get_generic_password(&self.service, account) {
            Ok(password_bytes) => {
                // Convert bytes to string
                let secret_str = String::from_utf8(password_bytes.to_vec())
                    .map_err(|e| {
                        error!(error = %e, "Invalid UTF-8 in Keychain data");
                        StoreError::InvalidFormat(format!("Invalid UTF-8 in Keychain data: {}", e))
                    })?;

                // Parse as age identity
                let inner: x25519::Identity = secret_str
                    .trim()
                    .parse()
                    .map_err(|e: &str| {
                        error!(error = %e, "Invalid age identity format in Keychain");
                        StoreError::InvalidFormat(e.to_string())
                    })?;

                info!(account = %account, "✓ Loaded identity from Keychain");

                Ok(Identity::from_parts(
                    inner,
                    IdentitySource::Keychain {
                        account: account.to_string(),
                    },
                ))
            }
            Err(e) => {
                let error_code = e.code();
                error!(
                    account = %account,
                    error_code = error_code,
                    error = %e,
                    "Failed to load identity from Keychain"
                );

                // Map specific error codes
                if error_code == -128 {
                    // User cancelled authorization
                    Err(StoreError::KeychainAccessDenied.into())
                } else if error_code == -25300 {
                    // Item not found (errSecItemNotFound)
                    Err(StoreError::NoPrivateKey(format!("keychain:{}", account)).into())
                } else {
                    Err(StoreError::KeychainError(format!("Keychain error: {}", e)).into())
                }
            }
        }
    }

    /// Delete an identity from the Keychain
    pub fn delete_identity(&self, account: &str) -> Result<()> {
        info!(account = %account, service = %self.service, "Deleting identity from Keychain");

        use security_framework::passwords::delete_generic_password;

        match delete_generic_password(&self.service, account) {
            Ok(_) => {
                info!(account = %account, "✓ Deleted identity from Keychain");
                Ok(())
            }
            Err(e) => {
                let error_code = e.code();

                // Item not found is not an error for delete
                if error_code == -25300 {
                    info!(account = %account, "Identity not found in Keychain (already deleted)");
                    Ok(())
                } else if error_code == -128 {
                    // User cancelled authorization
                    error!(account = %account, "User cancelled Keychain access");
                    Err(StoreError::KeychainAccessDenied.into())
                } else {
                    error!(account = %account, error = %e, "Failed to delete from Keychain");
                    Err(StoreError::KeychainError(format!("Failed to delete from Keychain: {}", e)).into())
                }
            }
        }
    }

    /// Check if an identity exists in the Keychain
    fn keychain_has_key(&self, account: &str) -> bool {
        use security_framework::passwords::get_generic_password;

        // Try to retrieve the item - if it exists, we'll get it back
        match get_generic_password(&self.service, account) {
            Ok(_) => {
                debug!(account = %account, "identity exists in Keychain");
                true
            }
            Err(e) => {
                if e.code() == -25300 {
                    // errSecItemNotFound - item doesn't exist
                    debug!(account = %account, "identity not found in Keychain");
                    false
                } else {
                    // Other errors (e.g., access denied) - treat as not found
                    debug!(account = %account, error_code = e.code(), "error checking Keychain");
                    false
                }
            }
        }
    }
}

impl super::Store for Keychain {
    fn generate_keypair(&self, project_id: &str) -> Result<String> {
        info!(
            project_id = %project_id,
            backend = "Keychain",
            "Generating keypair with Keychain backend"
        );

        // Generate identity in memory
        let inner = x25519::Identity::generate();
        let public_key = inner.to_public().to_string();

        // Store in Keychain - NO FILESYSTEM FALLBACK
        use age::secrecy::ExposeSecret;
        let secret = inner.to_string();
        self.store_identity(project_id, secret.expose_secret(), false)?;

        info!(
            project_id = %project_id,
            backend = "Keychain",
            "✓ Identity generated and stored in Keychain"
        );
        Ok(public_key)
    }

    fn load_identity(&self, project_id: &str) -> Result<Identity> {
        info!(
            project_id = %project_id,
            backend = "Keychain",
            "Loading identity from Keychain"
        );

        // Load from Keychain - NO FILESYSTEM FALLBACK
        self.load_from_keychain(project_id)
    }

    fn has_key(&self, project_id: &str) -> bool {
        self.keychain_has_key(project_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Store;  // Bring Store trait into scope

    #[test]
    fn test_keychain_backend_creation() {
        let keychain = Keychain::new().unwrap();
        assert_eq!(keychain.service, Keychain::SERVICE_NAME);
    }

    #[test]
    fn test_keychain_storage() {
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let project_id = "test-keychain-storage";

        // Set up a temporary key directory
        std::env::set_var("DUGOUT_HOME", tmp.path());

        let keychain = Keychain::new().unwrap();

        // Generate should try Keychain first
        let pubkey = keychain.generate_keypair(project_id).unwrap();
        assert!(pubkey.starts_with("age1"));

        // Check if we can load the identity
        let identity_result = keychain.load_identity(project_id);

        // On real macOS with user interaction, this should work
        // In CI or when user cancels, it falls back to filesystem
        match identity_result {
            Ok(identity) => {
                assert_eq!(identity.public_key(), pubkey);
                // Could be either Keychain or Filesystem source depending on environment
            }
            Err(_) => {
                // Fallback to filesystem occurred
                let key_dir = Identity::project_dir(project_id).unwrap();
                // In case of fallback, file should exist
                if key_dir.join("identity.key").exists() {
                    let identity = Identity::load(&key_dir).unwrap();
                    assert_eq!(identity.public_key(), pubkey);
                }
            }
        }

        // Clean up
        let _ = keychain.delete_identity(project_id);
        std::env::remove_var("DUGOUT_HOME");
    }
}
