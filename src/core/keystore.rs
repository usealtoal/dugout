//! Key generation and storage.
//!
//! Manages age identity (private key) generation and retrieval per project.

use std::fs;
use std::path::PathBuf;

use crate::error::{KeyError, Result};

/// Key storage manager for age identities.
pub struct KeyStore;

impl KeyStore {
    /// Base directory for all burrow keys (`~/.burrow/keys`).
    fn base_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".burrow")
            .join("keys")
    }

    /// Directory for a specific project's keys.
    fn project_dir(project_id: &str) -> PathBuf {
        Self::base_dir().join(project_id)
    }

    /// Generate a new age keypair for a project.
    ///
    /// Creates the key directory if it doesn't exist and stores the private
    /// key with restricted permissions (0600 on Unix).
    ///
    /// # Arguments
    ///
    /// * `project_id` - Unique identifier for the project
    ///
    /// # Returns
    ///
    /// The public key string (starts with "age1...").
    ///
    /// # Errors
    ///
    /// Returns `KeyError` if key generation or file operations fail.
    pub fn generate_keypair(project_id: &str) -> Result<String> {
        let identity = age::x25519::Identity::generate();
        let public_key = identity.to_public().to_string();

        let dir = Self::project_dir(project_id);
        fs::create_dir_all(&dir)
            .map_err(KeyError::WriteFailed)?;

        let key_path = dir.join("identity.key");

        // Write identity using Display trait (outputs AGE-SECRET-KEY-...)
        use age::secrecy::ExposeSecret;
        let secret_str = identity.to_string();
        fs::write(&key_path, format!("{}\n", secret_str.expose_secret()))
            .map_err(KeyError::WriteFailed)?;

        // Restrict permissions on key file (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600))
                .map_err(KeyError::WriteFailed)?;
        }

        Ok(public_key)
    }

    /// Load the private key (identity) for a project.
    ///
    /// # Arguments
    ///
    /// * `project_id` - Unique identifier for the project
    ///
    /// # Returns
    ///
    /// The age x25519 identity for decryption.
    ///
    /// # Errors
    ///
    /// Returns `KeyError::NoPrivateKey` if the key doesn't exist,
    /// or `KeyError::InvalidFormat` if the key is malformed.
    pub fn load_identity(project_id: &str) -> Result<age::x25519::Identity> {
        let key_path = Self::project_dir(project_id).join("identity.key");
        if !key_path.exists() {
            return Err(KeyError::NoPrivateKey(project_id.to_string()).into());
        }

        let contents = fs::read_to_string(&key_path)
            .map_err(KeyError::ReadFailed)?;
        
        let identity: age::x25519::Identity = contents
            .trim()
            .parse()
            .map_err(|e| KeyError::InvalidFormat(format!("{}", e)))?;

        Ok(identity)
    }

    /// Check if a keypair exists for a project.
    ///
    /// # Arguments
    ///
    /// * `project_id` - Unique identifier for the project
    ///
    /// # Returns
    ///
    /// `true` if an identity key file exists, `false` otherwise.
    pub fn has_key(project_id: &str) -> bool {
        Self::project_dir(project_id).join("identity.key").exists()
    }
}
