//! Core library components.
//!
//! This module contains the reusable business logic for secret management,
//! encryption, and configuration handling.

// Public API
pub mod recipient;
pub mod types;
pub mod vault;

// Internal implementation - exposed to CLI but not public API
pub(crate) mod cipher;
pub(crate) mod config;
pub(crate) mod constants;
pub(crate) mod env;
pub(crate) mod secrets;
pub(crate) mod store;
pub(crate) mod team;
