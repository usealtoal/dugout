//! Burrow - An extremely fast secrets manager for developers.
//!
//! # Architecture
//!
//! ```text
//! src/
//! ├── core/             # Core library components
//! │   ├── crypto/       # Encryption/decryption logic
//! │   ├── config/       # Configuration management
//! │   ├── keystore/     # Key generation and storage
//! │   └── secrets/      # Secret operations
//! └── cli/              # Command-line interface
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
