//! Filesystem-based key storage implementation.
//!
//! Manages age identity (private key) generation and retrieval from
//! the local filesystem (~/.burrow/keys/).

use super::Store;
use crate::core::identity::Identity;
use crate::error::Result;

/// Filesystem-based key storage.
///
/// Stores age identities in `~/.burrow/keys/<project_id>/identity.key`.
pub struct Filesystem;

impl Store for Filesystem {
    fn generate_keypair(&self, project_id: &str) -> Result<String> {
        let key_dir = Identity::project_dir(project_id)?;
        let identity = Identity::generate(&key_dir)?;
        Ok(identity.public_key())
    }

    fn load_identity(&self, project_id: &str) -> Result<Identity> {
        let key_dir = Identity::project_dir(project_id)?;
        Identity::load(&key_dir)
    }

    fn has_key(&self, project_id: &str) -> bool {
        Identity::project_dir(project_id)
            .map(|dir| dir.join("identity.key").exists())
            .unwrap_or(false)
    }
}
