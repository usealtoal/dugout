use thiserror::Error;

#[derive(Error, Debug)]
pub enum BurrowError {
    #[error("not initialized: run `burrow init` first")]
    NotInitialized,

    #[error("already initialized: .burrow.toml exists")]
    AlreadyInitialized,

    #[error("secret not found: {0}")]
    SecretNotFound(String),

    #[error("secret already exists: {0} (use --force to overwrite)")]
    SecretExists(String),

    #[error("no private key found for this project")]
    NoPrivateKey,

    #[error("decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("invalid key: {0}")]
    InvalidKey(String),

    #[error("recipient not found: {0}")]
    RecipientNotFound(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("toml serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
}

pub type Result<T> = std::result::Result<T, BurrowError>;
