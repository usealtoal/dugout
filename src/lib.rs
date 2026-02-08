//! Burrow - An extremely fast secrets manager for developers.
//!
//! # Architecture
//!
//! ```text
//! src/
//! ├── cli/              # Command-line interface
//! │   ├── init          # Initialize burrow
//! │   ├── secrets       # Secret CRUD operations
//! │   ├── lock          # Lock/unlock commands
//! │   ├── run           # Run with injected secrets
//! │   ├── shell         # Interactive shell with secrets
//! │   ├── team          # Team management
//! │   ├── env           # .env import/export/diff
//! │   ├── status        # Quick status overview
//! │   ├── audit         # Git history security audit
//! │   └── completions   # Shell completions
//! └── core/             # Core library components
//!     ├── vault         # Main API - Vault struct
//!     ├── domain/       # Domain types
//!     │   ├── secret    # Secret
//!     │   ├── recipient # Recipient
//!     │   ├── identity  # Identity
//!     │   ├── diff      # Diff, DiffEntry, EntryStatus
//!     │   ├── env       # Env
//!     │   └── audit     # Finding, Severity
//!     ├── types         # Domain type aliases
//!     ├── config        # .burrow.toml management
//!     ├── cipher/       # Encryption backends
//!     │   ├── mod       # Cipher trait
//!     │   └── age       # age encryption implementation
//!     └── store/        # Key storage backends
//!         ├── mod       # Store trait
//!         └── fs        # Filesystem storage implementation
//! ```
//!
//! # Features
//!
//! - Age-based encryption with x25519 keys
//! - Team collaboration with multiple recipients
//! - Fast encrypted secret storage
//! - Seamless .env file integration
//! - Extensible crypto and storage backends
//!
//! # Public API
//!
//! The primary entry point is the [`Vault`] struct, which provides all
//! secret management and team collaboration operations.
//!
//! ```rust,no_run
//! use burrow::Vault;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize a new vault with default age cipher
//! let mut vault = Vault::init("alice", None, None, None)?;
//!
//! // Set a secret
//! vault.set("DATABASE_URL", "postgres://localhost/db", false)?;
//!
//! // Get a secret
//! let value = vault.get("DATABASE_URL")?;
//!
//! // Add a team member
//! vault.add_recipient("bob", "age1...")?;
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
