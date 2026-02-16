//! Filesystem-based key storage implementation.
//!
//! Manages age identity (private key) generation and retrieval from
//! the local filesystem (~/.dugout/keys/).

use super::Store;
use crate::core::domain::Identity;
use crate::error::Result;
use tracing::info;

/// Filesystem-based key storage.
///
/// Stores age identities in `~/.dugout/keys/<project_id>/identity.key`.
pub struct Filesystem;

impl Store for Filesystem {
    fn generate_keypair(&self, project_id: &str) -> Result<String> {
        info!(
            project_id = %project_id,
            backend = "Filesystem",
            "Generating keypair with Filesystem backend"
        );

        // Special handling for global identity
        if project_id == "global" {
            let identity = Identity::generate_global_filesystem_only()?;
            info!(
                project_id = %project_id,
                backend = "Filesystem",
                "✓ Global identity generated and stored in filesystem"
            );
            return Ok(identity.public_key());
        }

        let key_dir = Identity::project_dir(project_id)?;
        let identity = Identity::generate(&key_dir)?;

        info!(
            project_id = %project_id,
            backend = "Filesystem",
            path = %key_dir.display(),
            "✓ Identity generated and stored in filesystem"
        );

        Ok(identity.public_key())
    }

    fn load_identity(&self, project_id: &str) -> Result<Identity> {
        info!(
            project_id = %project_id,
            backend = "Filesystem",
            "Loading identity from filesystem"
        );

        // Special handling for global identity
        if project_id == "global" {
            let identity = Identity::load_global_filesystem_only()?;
            info!(
                project_id = %project_id,
                backend = "Filesystem",
                "✓ Global identity loaded from filesystem"
            );
            return Ok(identity);
        }

        let key_dir = Identity::project_dir(project_id)?;
        let identity = Identity::load(&key_dir)?;

        info!(
            project_id = %project_id,
            backend = "Filesystem",
            "✓ Loaded identity from filesystem"
        );

        Ok(identity)
    }

    fn has_key(&self, project_id: &str) -> bool {
        // Special handling for global identity
        if project_id == "global" {
            return Identity::global_path()
                .map(|p| p.exists())
                .unwrap_or(false);
        }

        Identity::project_dir(project_id)
            .map(|dir| dir.join("identity.key").exists())
            .unwrap_or(false)
    }
}
