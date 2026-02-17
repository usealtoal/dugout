//! Key management operations.
//!
//! Provides key generation and storage abstraction with implementations
//! for different storage backends.
//!
//! ## Adding a New Storage Backend
//!
//! 1. Implement the `Store` trait
//! 2. Add the implementation in a new file (e.g., `cloud.rs`, `vault.rs`)
//! 3. Re-export from this module
//!
//! ## Example
//!
//! ```ignore
//! struct Cloud { /* ... */ }
//!
//! impl Store for Cloud {
//!     fn generate_keypair(&self, project_id: &str) -> Result<String> {
//!         // Generate and store in cloud
//!     }
//!     fn load_identity(&self, project_id: &str) -> Result<Identity> {
//!         // Load from cloud
//!     }
//!     fn has_key(&self, project_id: &str) -> bool {
//!         // Check cloud storage
//!     }
//! }
//! ```

use crate::core::domain::Identity;
use crate::error::Result;

mod backend;
mod fs;

#[cfg(target_os = "macos")]
pub mod keychain;

pub use backend::default_backend;
pub use fs::Filesystem;

/// Key storage trait.
///
/// Abstracts key generation and retrieval to support multiple
/// storage backends (filesystem, cloud KMS, vault, etc.).
pub trait Store {
    /// Generate a new keypair for a project.
    ///
    /// # Arguments
    ///
    /// * `project_id` - Unique identifier for the project
    ///
    /// # Returns
    ///
    /// The public key string.
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if key generation or storage fails.
    fn generate_keypair(&self, project_id: &str) -> Result<String>;

    /// Load the private key (identity) for a project.
    ///
    /// # Arguments
    ///
    /// * `project_id` - Unique identifier for the project
    ///
    /// # Returns
    ///
    /// The private key/identity for decryption.
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the key doesn't exist or cannot be loaded.
    fn load_identity(&self, project_id: &str) -> Result<Identity>;

    /// Check if a keypair exists for a project.
    ///
    /// # Arguments
    ///
    /// * `project_id` - Unique identifier for the project
    ///
    /// # Returns
    ///
    /// `true` if a key exists, `false` otherwise.
    fn has_key(&self, project_id: &str) -> bool;
}

/// Generate a new age keypair for a project.
///
/// Creates the key directory if it doesn't exist and stores the private
/// key with restricted permissions (0600 on Unix).
///
/// On macOS, stores in Keychain by default (or filesystem if Keychain is disabled).
/// On other platforms, stores in filesystem.
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
/// Returns `StoreError` if key generation or file operations fail.
pub fn generate_keypair(project_id: &str) -> Result<String> {
    default_backend().generate_keypair(project_id)
}

/// Load the private key (identity) for a project.
///
/// On macOS, tries Keychain first, then falls back to filesystem.
/// On other platforms, loads from filesystem.
///
/// # Arguments
///
/// * `project_id` - Unique identifier for the project
///
/// # Returns
///
/// The Identity for decryption.
///
/// # Errors
///
/// Returns `StoreError::NoPrivateKey` if the key doesn't exist,
/// or `StoreError::InvalidFormat` if the key is malformed.
pub fn load_identity(project_id: &str) -> Result<Identity> {
    default_backend().load_identity(project_id)
}

/// Check if a keypair exists for a project.
///
/// On macOS, checks Keychain then filesystem.
/// On other platforms, checks filesystem.
///
/// # Arguments
///
/// * `project_id` - Unique identifier for the project
///
/// # Returns
///
/// `true` if an identity key exists, `false` otherwise.
pub fn has_key(project_id: &str) -> bool {
    default_backend().has_key(project_id)
}

/// Check if the global identity exists in the active backend or filesystem.
pub fn has_global() -> Result<bool> {
    if has_key("global") {
        return Ok(true);
    }
    Identity::has_global()
}

/// Load global identity from active backend, then filesystem fallback.
pub fn load_global_identity() -> Result<Identity> {
    load_identity("global").or_else(|_| Identity::load_global())
}
