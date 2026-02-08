//! Secret type.
//!
//! Represents a single encrypted secret with its key and ciphertext.

use crate::core::types::{EncryptedValue, SecretKey};

/// An encrypted secret with its key name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Secret {
    key: SecretKey,
    value: EncryptedValue,
}

impl Secret {
    /// Create a new secret from a key and encrypted value
    pub fn new(key: SecretKey, value: EncryptedValue) -> Self {
        Self { key, value }
    }

    /// Secret's key name
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Encrypted ciphertext
    pub fn encrypted(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_new() {
        let secret = Secret::new("API_KEY".to_string(), "age-encryption-v1...".to_string());

        assert_eq!(secret.key(), "API_KEY");
        assert_eq!(secret.encrypted(), "age-encryption-v1...");
    }

    #[test]
    fn test_secret_display() {
        let secret = Secret::new("DATABASE_URL".to_string(), "encrypted_value".to_string());

        assert_eq!(format!("{}", secret), "DATABASE_URL");
    }
}
