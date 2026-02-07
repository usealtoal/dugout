use std::fs;
use std::path::PathBuf;

use crate::error::{BurrowError, Result};

pub struct KeyStore;

impl KeyStore {
    /// Base directory for all burrow keys
    fn base_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".burrow")
            .join("keys")
    }

    /// Directory for a specific project's keys
    fn project_dir(project_id: &str) -> PathBuf {
        Self::base_dir().join(project_id)
    }

    /// Generate a new age keypair, returning (public_key, identity_string)
    pub fn generate_keypair(project_id: &str) -> Result<String> {
        let identity = age::x25519::Identity::generate();
        let public_key = identity.to_public().to_string();

        let dir = Self::project_dir(project_id);
        fs::create_dir_all(&dir)?;

        let key_path = dir.join("identity.key");

        // Write identity using Display trait (outputs AGE-SECRET-KEY-...)
        use age::secrecy::ExposeSecret;
        let secret_str = identity.to_string();
        fs::write(&key_path, format!("{}\n", secret_str.expose_secret()))?;

        // Restrict permissions on key file (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(public_key)
    }

    /// Load the private key for a project
    pub fn load_identity(project_id: &str) -> Result<age::x25519::Identity> {
        let key_path = Self::project_dir(project_id).join("identity.key");
        if !key_path.exists() {
            return Err(BurrowError::NoPrivateKey);
        }

        let contents = fs::read_to_string(&key_path)?;
        let identity: age::x25519::Identity = contents
            .trim()
            .parse()
            .map_err(|e| BurrowError::InvalidKey(format!("{}", e)))?;

        Ok(identity)
    }

    /// Check if a keypair exists for a project
    pub fn has_key(project_id: &str) -> bool {
        Self::project_dir(project_id).join("identity.key").exists()
    }
}
