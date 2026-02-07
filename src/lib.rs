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
//!     ├── crypto        # age encryption/decryption
//!     ├── env           # .env file operations
//!     ├── keys          # Key generation and storage
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

pub mod cli;
pub mod core;
pub mod error;
