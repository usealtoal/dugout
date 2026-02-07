//! Identity type.
//!
//! Wraps an age private key with secure memory handling.

use std::fs;
use std::path::{Path, PathBuf};

use age::x25519;
use tracing::{debug, warn};

use crate::core::constants;
use crate::core::types::PublicKey;
use crate::error::{Result, StoreError, ValidationError};

/// A private key identity for decrypting secrets.
pub struct Identity {
    inner: x25519::Identity,
    path: PathBuf,
}

impl Identity {
    /// Load an identity from the key directory.
    pub fn load(key_dir: &Path) -> Result<Self> {
        let key_path = key_dir.join("identity.key");
        debug!("Loading identity from: {}", key_path.display());

        if !key_path.exists() {
            return Err(StoreError::NoPrivateKey(key_dir.display().to_string()).into());
        }

        // Verify permissions on Unix
        #[cfg(unix)]
        {
            if let Err(e) = Self::validate_file_permissions(&key_path, 0o600) {
                warn!(
                    "Insecure key file permissions: {}. Run: chmod 600 {}",
                    e,
                    key_path.display()
                );
            }
        }

        let contents = fs::read_to_string(&key_path).map_err(StoreError::ReadFailed)?;

        let inner: x25519::Identity = contents
            .trim()
            .parse()
            .map_err(|e: &str| StoreError::InvalidFormat(e.to_string()))?;

        debug!("Identity loaded successfully");

        Ok(Self {
            inner,
            path: key_path,
        })
    }

    /// Generate a new identity and save to disk.
    pub fn generate(key_dir: &Path) -> Result<Self> {
        debug!("Generating new identity in: {}", key_dir.display());

        let inner = x25519::Identity::generate();

        fs::create_dir_all(key_dir).map_err(StoreError::WriteFailed)?;

        let key_path = key_dir.join("identity.key");

        // Write identity using Display trait (outputs AGE-SECRET-KEY-...)
        use age::secrecy::ExposeSecret;
        let secret_str = inner.to_string();
        fs::write(&key_path, format!("{}\n", secret_str.expose_secret()))
            .map_err(StoreError::WriteFailed)?;

        // Restrict permissions on key file (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600))
                .map_err(StoreError::WriteFailed)?;
        }

        debug!("Identity generated and saved to: {}", key_path.display());

        Ok(Self {
            inner,
            path: key_path,
        })
    }

    /// Get the corresponding public key.
    pub fn public_key(&self) -> PublicKey {
        self.inner.to_public().to_string()
    }

    /// Get a reference to the inner age identity (for decryption).
    pub fn as_age(&self) -> &x25519::Identity {
        &self.inner
    }

    /// Get the key file path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Base directory for all burrow keys (`~/.burrow/keys`).
    fn base_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            StoreError::GenerationFailed("unable to determine home directory".to_string())
        })?;
        Ok(home.join(constants::KEY_DIR))
    }

    /// Directory for a specific project's keys.
    pub fn project_dir(project_id: &str) -> Result<PathBuf> {
        Ok(Self::base_dir()?.join(project_id))
    }

    /// Validate file permissions (Unix only).
    #[cfg(unix)]
    fn validate_file_permissions(path: &Path, expected_mode: u32) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let metadata = fs::metadata(path)?;
        let actual_mode = metadata.permissions().mode() & 0o777;

        if actual_mode != expected_mode {
            return Err(ValidationError::InvalidPermissions {
                path: path.display().to_string(),
                expected: format!("{:o}", expected_mode),
                actual: format!("{:o}", actual_mode),
            }
            .into());
        }

        Ok(())
    }
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identity")
            .field("path", &self.path)
            .field("public_key", &self.public_key())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_and_load() {
        let tmp = TempDir::new().unwrap();
        let key_dir = tmp.path().join("test-project");

        // Generate a new identity
        let identity1 = Identity::generate(&key_dir).unwrap();
        let pubkey1 = identity1.public_key();

        // Load the same identity
        let identity2 = Identity::load(&key_dir).unwrap();
        let pubkey2 = identity2.public_key();

        // Should have the same public key
        assert_eq!(pubkey1, pubkey2);
    }

    #[test]
    fn test_public_key() {
        let tmp = TempDir::new().unwrap();
        let key_dir = tmp.path().join("test-project");

        let identity = Identity::generate(&key_dir).unwrap();
        let pubkey = identity.public_key();

        // age public keys start with "age1"
        assert!(pubkey.starts_with("age1"));
    }

    #[test]
    fn test_path() {
        let tmp = TempDir::new().unwrap();
        let key_dir = tmp.path().join("test-project");

        let identity = Identity::generate(&key_dir).unwrap();
        let path = identity.path();

        assert!(path.exists());
        assert!(path.ends_with("identity.key"));
    }
}
