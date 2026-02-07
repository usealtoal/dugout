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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipient_new_valid() {
        let identity = age::x25519::Identity::generate();
        let pubkey = identity.to_public().to_string();

        let recipient = Recipient::new("alice".to_string(), pubkey.clone());
        assert!(recipient.is_ok());

        let recipient = recipient.unwrap();
        assert_eq!(recipient.name(), "alice");
        assert_eq!(recipient.public_key(), &pubkey);
    }

    #[test]
    fn test_recipient_new_invalid_key() {
        let result = Recipient::new("bob".to_string(), "not-a-valid-key".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_recipient_new_empty_name() {
        let identity = age::x25519::Identity::generate();
        let pubkey = identity.to_public().to_string();

        let recipient = Recipient::new("".to_string(), pubkey);
        // Empty names are technically allowed by the type system
        // but may fail validation elsewhere - this should succeed at the Recipient level
        assert!(recipient.is_ok());
    }
}
