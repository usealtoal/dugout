//! Cryptographic operations.
//!
//! Three backends:
//! - **age** (default): x25519 public-key encryption
//! - **hybrid**: age + cloud KMS (AWS or GCP)
//! - **gpg**: GPG encryption via CLI (feature-gated)
//!
//! Cloud KMS providers live in `provider/`.

use crate::error::Result;

mod age;
mod backend;
pub mod provider;

#[cfg(feature = "gpg")]
pub mod gpg;

pub use age::{parse_recipient, Age};
pub use backend::CipherBackend;
#[allow(unused_imports)]
pub use provider::{Envelope, KmsProvider};

/// Cryptographic backend trait.
pub trait Cipher {
    type Recipient;
    type Identity;

    fn encrypt(&self, plaintext: &str, recipients: &[Self::Recipient]) -> Result<String>;
    fn decrypt(&self, encrypted: &str, identity: &Self::Identity) -> Result<String>;
    fn name(&self) -> &'static str;
}
