//! Type aliases for domain concepts.
//!
//! Provides semantic type aliases to make function signatures more descriptive.

/// A secret key name (e.g., DATABASE_URL, API_KEY).
///
/// Must be a valid environment variable name.
pub type SecretKey = String;

/// An encrypted secret value (age-armored ciphertext).
///
/// Contains the age-encrypted and ASCII-armored representation of a secret.
pub type EncryptedValue = String;

/// An age public key string (starts with "age1...").
///
/// Used for encrypting secrets for specific recipients.
pub type PublicKey = String;

/// A team member name.
///
/// Identifies a recipient in the dugout configuration.
pub type MemberName = String;
