//! A local secrets manager for development teams
//!
//! Dugout encrypts secrets at rest using pluggable cipher backends
//! (age, AWS KMS, GCP KMS, GPG) and provides a simple CLI for
//! managing secrets across teams.
//!
//! # Quick start
//!
//! ```no_run
//! use dugout::Vault;
//!
//! let mut vault = Vault::open()?;
//! vault.set("DATABASE_URL", "postgres://localhost/db", false)?;
//! let value = vault.get("DATABASE_URL")?;
//! # Ok::<(), dugout::error::Error>(())
//! ```
//!
//! # Architecture
//!
//! The crate is organized into two main modules:
//!
//! - **`core`**: Library code with [`Vault`] as the main entry point
//! - **`cli`**: Command-line interface and user-facing commands
//!
//! ## Core Components
//!
//! - [`Vault`]: Main API for all secret operations
//! - Domain types: [`Secret`], [`Recipient`], [`Identity`], [`Env`], [`Diff`]
//! - Pluggable cipher backends (age, AWS KMS, GCP KMS, GPG)
//! - Configuration in `.dugout.toml`
//!
//! # Features
//!
//! - **Fast**: Age encryption with x25519 keys
//! - **Team-ready**: Multiple recipients, key rotation
//! - **Flexible**: Pluggable cipher backends (age, KMS, GPG)
//! - **Developer-friendly**: `.env` file integration, shell completion
//! - **Secure**: No secrets in git history, encrypted at rest
//!
//! # Example: Initialize and use a vault
//!
//! ```rust,no_run
//! use dugout::Vault;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize a new vault with default age cipher
//! let mut vault = Vault::init("alice", None, None)?;
//!
//! // Set a secret
//! vault.set("DATABASE_URL", "postgres://localhost/db", false)?;
//!
//! // Get a secret
//! let value = vault.get("DATABASE_URL")?;
//!
//! // Add a team member
//! vault.add_recipient("bob", "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p")?;
//!
//! // List all secrets
//! for secret in vault.list() {
//!     println!("{}", secret.key());
//! }
//! # Ok(())
//! # }
//! ```

pub mod cli;
pub mod core;
pub mod error;

// Re-export the public API
pub use core::domain::*;
pub use core::types::*;
pub use core::vault::Vault;

/// Benchmark support: re-export cipher and config internals.
#[doc(hidden)]
pub mod bench {
    pub use crate::core::cipher::{Age, Cipher};
    pub use crate::core::config::Config;
}

/// Test-only exports for KMS integration tests.
#[cfg(any(feature = "test-aws", feature = "test-gcp"))]
#[doc(hidden)]
pub mod test {
    #[cfg(feature = "test-aws")]
    pub use crate::core::cipher::aws;
    #[cfg(feature = "test-gcp")]
    pub use crate::core::cipher::gcp;
    pub use crate::core::cipher::Cipher;
}
