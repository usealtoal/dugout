//! Cloud KMS backends for hybrid encryption.
//!
//! Shared types (Envelope, KmsProvider, KmsBackend) plus provider implementations.
//!
//! - `aws`: AWS KMS (feature-gated)
//! - `gcp`: GCP Cloud KMS (feature-gated)

pub mod envelope;
pub use envelope::{Envelope, KmsProvider};

#[cfg(feature = "aws")]
pub mod aws;

#[cfg(feature = "gcp")]
pub mod gcp;
