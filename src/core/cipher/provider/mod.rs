//! Cloud KMS providers for hybrid encryption.
//!
//! - `kms`: shared types (Envelope, KmsProvider, KmsBackend)
//! - `aws`: AWS KMS implementation (feature-gated)
//! - `gcp`: GCP Cloud KMS implementation (feature-gated)

pub mod kms;

#[cfg(feature = "aws")]
pub mod aws;

#[cfg(feature = "gcp")]
pub mod gcp;

pub use kms::{Envelope, KmsProvider};
