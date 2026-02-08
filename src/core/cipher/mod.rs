//! Cryptographic operations.
//!
//! Provides encryption/decryption abstraction and implementations.
//! Supports multiple backends: age (default), AWS KMS, GCP KMS, and GPG.
//!
//! ## Backends
//!
//! - **age**: Default, always available. Uses x25519 public-key encryption.
//! - **AWS KMS**: Feature-gated (`aws`). Uses AWS Key Management Service.
//! - **GCP KMS**: Feature-gated (`gcp`). Uses Google Cloud KMS via gcloud CLI.
//! - **GPG**: Feature-gated (`gpg`). Uses GnuPG via gpg CLI.
//!
//! ## Adding a New Backend
//!
//! 1. Implement the `Cipher` trait
//! 2. Add the implementation in a new file (e.g., `kms.rs`, `gpg.rs`)
//! 3. Feature-gate if appropriate
//! 4. Re-export from this module

use crate::error::Result;

mod age;
mod backend;

#[cfg(feature = "aws")]
pub mod aws;

#[cfg(feature = "gcp")]
pub mod gcp;

#[cfg(feature = "gpg")]
pub mod gpg;

pub use age::{parse_recipient, Age};
pub use backend::CipherBackend;

/// Cryptographic backend trait
///
/// Abstracts encryption and decryption operations to support
/// multiple cryptographic backends (age, KMS, GPG, etc.).
///
/// Recipients are backend-specific:
/// - age: public keys (age1...)
/// - AWS KMS: key ARNs or IDs
/// - GCP KMS: resource names (projects/.../cryptoKeys/...)
/// - GPG: key fingerprints or email addresses
pub trait Cipher {
    /// Type representing a recipient public key
    type Recipient;

    /// Type representing a private identity/key
    type Identity;

    /// Encrypt plaintext for multiple recipients
    ///
    /// # Errors
    ///
    /// Returns `CipherError` if encryption fails.
    fn encrypt(&self, plaintext: &str, recipients: &[Self::Recipient]) -> Result<String>;

    /// Decrypt an encrypted string using a private identity
    ///
    /// # Errors
    ///
    /// Returns `CipherError` if decryption fails.
    fn decrypt(&self, encrypted: &str, identity: &Self::Identity) -> Result<String>;

    /// Backend name for display/config
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
}

// Direct encrypt/decrypt helpers removed — use CipherBackend for all operations.
// Age types (Identity, Recipient) accessed directly via age::x25519 where needed.
