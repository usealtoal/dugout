//! Core library components.
//!
//! This module contains the reusable business logic for secret management,
//! encryption, and configuration handling.

// Public API
pub mod domain;
pub mod types;
pub mod vault;

// Internal implementation - exposed to CLI but not public API
pub(crate) mod cipher;
pub(crate) mod config;
pub(crate) mod constants;
pub(crate) mod detect;
pub(crate) mod store;
