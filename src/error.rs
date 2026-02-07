//! Error types for Burrow.
//!
//! Domain-specific error types following best practices.

use thiserror::Error;

/// Configuration-related errors.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("not initialized\n  → Run `burrow init` to get started")]
    NotInitialized,

    #[error("already initialized: .burrow.toml exists")]
    AlreadyInitialized,

    #[error("no recipients configured")]
    NoRecipients,

    #[error("recipient not found: {0}")]
    RecipientNotFound(String),

    #[error("missing required field: {field}")]
    MissingField { field: &'static str },

    #[error("invalid value for {field}: {reason}")]
    InvalidValue { field: &'static str, reason: String },

    #[error("failed to read config file: {0}")]
    ReadFile(#[source] std::io::Error),

    #[error("config file is malformed: {0}")]
    Parse(#[source] toml::de::Error),

    #[error("failed to serialize config: {0}")]
    Serialize(#[source] toml::ser::Error),

    #[error("{0}")]
    Other(String),
}

/// Cryptographic operation errors.
#[derive(Error, Debug)]
pub enum CipherError {
    #[error("encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("invalid age public key: {0}")]
    InvalidPublicKey(String),

    #[error("invalid age secret key: {0}")]
    InvalidSecretKey(String),

    #[error("armor encoding failed: {0}")]
    ArmorFailed(String),

    #[error("io error during crypto operation: {0}")]
    Io(#[source] std::io::Error),
}

/// Key storage and management errors.
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("no private key found for project '{0}'\n  → Ask a team member to share the project key, or run `burrow init` to start fresh")]
    NoPrivateKey(String),

    #[error("failed to generate keypair: {0}")]
    GenerationFailed(String),

    #[error("failed to read key file: {0}")]
    ReadFailed(#[source] std::io::Error),

    #[error("failed to write key file: {0}")]
    WriteFailed(#[source] std::io::Error),

    #[error("invalid key format: {0}")]
    InvalidFormat(String),
}

/// Secret operation errors.
#[derive(Error, Debug)]
pub enum SecretError {
    #[error("secret not found: {key}{suggestion}")]
    NotFound { key: String, suggestion: String },

    #[error("secret already exists: {0} (use --force to overwrite)")]
    AlreadyExists(String),

    #[error("failed to set secret: {0}")]
    SetFailed(String),

    #[error("failed to get secret: {0}")]
    GetFailed(String),

    #[error("failed to remove secret: {0}")]
    RemoveFailed(String),
}

impl SecretError {
    /// Create a NotFound error with suggestions based on available keys.
    pub fn not_found_with_suggestions(key: String, available_keys: &[String]) -> Self {
        let suggestion = if available_keys.is_empty() {
            "\n  → No secrets stored yet. Use `burrow set KEY VALUE` to add one".to_string()
        } else {
            let keys_list = available_keys.join(", ");
            format!(
                "\n  → Available keys: {}\n  → Did you mean one of these?",
                keys_list
            )
        };

        Self::NotFound { key, suggestion }
    }
}

/// Input validation errors.
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("invalid secret key '{key}': {reason}")]
    InvalidKey { key: String, reason: String },

    #[error("empty key is not allowed")]
    EmptyKey,

    #[error("empty value is not allowed for key '{0}'")]
    EmptyValue(String),

    #[error("invalid file permissions on '{path}': expected {expected}, got {actual}")]
    InvalidPermissions {
        path: String,
        expected: String,
        actual: String,
    },
}

/// Top-level Burrow error type.
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Cipher(#[from] CipherError),

    #[error(transparent)]
    Store(#[from] StoreError),

    #[error(transparent)]
    Secret(#[from] SecretError),

    #[error(transparent)]
    Validation(#[from] ValidationError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

/// Result type alias for Burrow operations.
pub type Result<T> = std::result::Result<T, Error>;
