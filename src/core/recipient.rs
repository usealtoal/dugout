//! Team member representation.
//!
//! Provides a validated type for team members who can decrypt secrets.

use crate::core::cipher;
use crate::core::types::{MemberName, PublicKey};
use crate::error::Result;

/// A team member who can decrypt secrets.
///
/// Represents a validated recipient with their name and age public key.
#[derive(Debug, Clone)]
pub struct Recipient {
    name: MemberName,
    public_key: PublicKey,
}

impl Recipient {
    /// Create a new recipient, validating the public key.
    ///
    /// # Arguments
    ///
    /// * `name` - Display name for the team member
    /// * `public_key` - age public key string (must be valid)
    ///
    /// # Errors
    ///
    /// Returns `CipherError` if the public key format is invalid.
    pub fn new(name: MemberName, public_key: PublicKey) -> Result<Self> {
        // Validate the key format
        cipher::parse_recipient(&public_key)?;

        Ok(Self { name, public_key })
    }

    /// Get the recipient's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the recipient's public key.
    pub fn public_key(&self) -> &str {
        &self.public_key
    }
}
