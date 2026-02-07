//! Cryptographic operations.
//!
//! Provides encryption/decryption abstraction and implementations.
//! Currently supports age encryption with plans for KMS/GPG backends.
//!
//! ## Adding a New Backend
//!
//! 1. Implement the `CryptoBackend` trait
//! 2. Add the implementation in a new file (e.g., `kms.rs`, `gpg.rs`)
//! 3. Re-export from this module
//!
//! ## Example
//!
//! ```ignore
//! struct KmsBackend { /* ... */ }
//!
//! impl CryptoBackend for KmsBackend {
//!     fn encrypt(&self, plaintext: &str, recipients: &[Recipient]) -> Result<String> {
//!         // KMS-specific encryption
//!     }
//!     fn decrypt(&self, encrypted: &str, identity: &Identity) -> Result<String> {
//!         // KMS-specific decryption
//!     }
//! }
//! ```

use crate::error::Result;
use ::age::x25519;

mod age;

pub use age::{parse_recipient, AgeBackend};

/// Cryptographic backend trait.
///
/// Abstracts encryption and decryption operations to support
/// multiple cryptographic backends (age, KMS, GPG, etc.).
pub trait CryptoBackend {
    /// Type representing a recipient public key.
    type Recipient;

    /// Type representing a private identity/key.
    type Identity;

    /// Encrypt plaintext for multiple recipients.
    ///
    /// # Arguments
    ///
    /// * `plaintext` - The string to encrypt
    /// * `recipients` - List of recipient public keys
    ///
    /// # Returns
    ///
    /// Encrypted string (format depends on backend implementation).
    ///
    /// # Errors
    ///
    /// Returns `CryptoError` if encryption fails.
    fn encrypt(&self, plaintext: &str, recipients: &[Self::Recipient]) -> Result<String>;

    /// Decrypt an encrypted string using a private identity.
    ///
    /// # Arguments
    ///
    /// * `encrypted` - Encrypted string
    /// * `identity` - Private key/identity
    ///
    /// # Returns
    ///
    /// The decrypted plaintext string.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError` if decryption fails.
    fn decrypt(&self, encrypted: &str, identity: &Self::Identity) -> Result<String>;
}

// Re-export commonly used age types for convenience
pub use ::age::x25519::{Identity, Recipient};

// Convenience functions using the default age backend
/// Encrypt plaintext for multiple age recipients.
///
/// This is a convenience wrapper around `AgeBackend::encrypt`.
///
/// # Arguments
///
/// * `plaintext` - The string to encrypt
/// * `recipients` - List of age public key recipients
///
/// # Returns
///
/// ASCII-armored encrypted string that any recipient can decrypt.
///
/// # Errors
///
/// Returns `CryptoError` if encryption fails at any stage.
pub fn encrypt(plaintext: &str, recipients: &[x25519::Recipient]) -> Result<String> {
    AgeBackend.encrypt(plaintext, recipients)
}

/// Decrypt an age-encrypted string using a private identity.
///
/// This is a convenience wrapper around `AgeBackend::decrypt`.
///
/// # Arguments
///
/// * `encrypted` - ASCII-armored encrypted string
/// * `identity` - age private key (x25519 identity)
///
/// # Returns
///
/// The decrypted plaintext string.
///
/// # Errors
///
/// Returns `CryptoError` if decryption fails or the key doesn't match.
pub fn decrypt(encrypted: &str, identity: &x25519::Identity) -> Result<String> {
    AgeBackend.decrypt(encrypted, identity)
}
