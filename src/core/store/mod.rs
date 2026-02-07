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

use crate::error::Result;
use age::x25519;

mod fs;

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
    fn load_identity(&self, project_id: &str) -> Result<x25519::Identity>;

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
    Filesystem.generate_keypair(project_id)
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
/// Returns `StoreError::NoPrivateKey` if the key doesn't exist,
/// or `StoreError::InvalidFormat` if the key is malformed.
pub fn load_identity(project_id: &str) -> Result<x25519::Identity> {
    Filesystem.load_identity(project_id)
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
    Filesystem.has_key(project_id)
}
