//! Filesystem-based key storage implementation.
//!
//! Manages age identity (private key) generation and retrieval from
//! the local filesystem (~/.burrow/keys/).

use std::fs;
use std::path::PathBuf;

use age::x25519;

use super::KeyStorage;
use crate::error::{KeyError, Result, ValidationError};

/// Validate file permissions (Unix only).
///
/// Checks that a file has the expected permissions mode.
#[cfg(unix)]
fn validate_file_permissions(path: &std::path::Path, expected_mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path)?;
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

/// Filesystem-based key storage.
///
/// Stores age identities in `~/.burrow/keys/<project_id>/identity.key`.
pub struct FilesystemKeyStore;

impl FilesystemKeyStore {
    /// Base directory for all burrow keys (`~/.burrow/keys`).
    fn base_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            KeyError::GenerationFailed("unable to determine home directory".to_string())
        })?;
        Ok(home.join(".burrow").join("keys"))
    }

    /// Directory for a specific project's keys.
    fn project_dir(project_id: &str) -> Result<PathBuf> {
        Ok(Self::base_dir()?.join(project_id))
    }
}

impl KeyStorage for FilesystemKeyStore {
    fn generate_keypair(&self, project_id: &str) -> Result<String> {
        let identity = x25519::Identity::generate();
        let public_key = identity.to_public().to_string();

        let dir = Self::project_dir(project_id)?;
        fs::create_dir_all(&dir).map_err(KeyError::WriteFailed)?;

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

    fn load_identity(&self, project_id: &str) -> Result<x25519::Identity> {
        let key_path = Self::project_dir(project_id)?.join("identity.key");
        if !key_path.exists() {
            return Err(KeyError::NoPrivateKey(project_id.to_string()).into());
        }

        // Verify permissions on Unix
        #[cfg(unix)]
        {
            if let Err(e) = validate_file_permissions(&key_path, 0o600) {
                eprintln!("Warning: {}", e);
                eprintln!("  Run: chmod 600 {}", key_path.display());
            }
        }

        let contents = fs::read_to_string(&key_path).map_err(KeyError::ReadFailed)?;

        let identity: x25519::Identity = contents
            .trim()
            .parse()
            .map_err(|e: &str| KeyError::InvalidFormat(e.to_string()))?;

        Ok(identity)
    }

    fn has_key(&self, project_id: &str) -> bool {
        Self::project_dir(project_id)
            .map(|dir| dir.join("identity.key").exists())
            .unwrap_or(false)
    }
}
