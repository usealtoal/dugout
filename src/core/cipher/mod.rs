//! Cryptographic operations.
//!
//! Two backends:
//! - **age** (default): x25519 public-key encryption
//! - **hybrid**: age + cloud KMS (AWS or GCP)
//!
//! KMS backends are feature-gated: `--features aws` or `--features gcp`.

use crate::error::Result;

mod age;
mod backend;
pub mod kms;

#[cfg(feature = "aws")]
pub mod aws;

#[cfg(feature = "gcp")]
pub mod gcp;

pub use age::{parse_recipient, Age};
pub use backend::CipherBackend;
#[allow(unused_imports)]
pub use kms::KmsProvider;

/// Cryptographic backend trait.
pub trait Cipher {
    type Recipient;
    type Identity;

    fn encrypt(&self, plaintext: &str, recipients: &[Self::Recipient]) -> Result<String>;
    fn decrypt(&self, encrypted: &str, identity: &Self::Identity) -> Result<String>;
    fn name(&self) -> &'static str;
}
