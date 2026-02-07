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
//! │   ├── team          # Team management
//! │   ├── env           # .env import/export/diff
//! │   └── completions   # Shell completions
//! └── core/             # Core library components
//!     ├── config        # .burrow.toml management
//!     ├── cipher/       # Encryption backends
//!     │   ├── mod       # Cipher trait
//!     │   └── age       # age encryption implementation
//!     ├── env           # .env file operations
//!     ├── store/         # Key storage backends
//!     │   ├── mod       # Store trait
//!     │   └── fs        # Filesystem storage implementation
//!     ├── secrets       # Secret CRUD logic
//!     └── team          # Team member management
//! ```
//!
//! # Features
//!
//! - Age-based encryption with x25519 keys
//! - Team collaboration with multiple recipients
//! - Fast encrypted secret storage
//! - Seamless .env file integration
//! - Extensible crypto and storage backends

pub mod cli;
pub mod core;
pub mod error;
