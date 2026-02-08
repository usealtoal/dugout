//! Shared KMS types, provider detection, and hybrid envelope.

use serde::{Deserialize, Serialize};

use crate::error::{CipherError, Result};

const ENVELOPE_V2: &str = "dugout-envelope-v2";

/// Supported KMS providers.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum KmsProvider {
    Aws,
    Gcp,
}

#[allow(dead_code)]
impl KmsProvider {
    /// Auto-detect provider from a key identifier.
    ///
    /// - `arn:aws:kms:...` → AWS
    /// - `projects/.../cryptoKeys/...` → GCP
    pub fn detect(key: &str) -> Option<Self> {
        if key.starts_with("arn:aws:kms:") {
            return Some(Self::Aws);
        }
        if key.starts_with("projects/") && key.contains("/cryptoKeys/") {
            return Some(Self::Gcp);
        }
        None
    }

    /// Provider display name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Aws => "aws",
            Self::Gcp => "gcp",
        }
    }
}

/// Hybrid envelope containing both age and KMS ciphertext.
///
/// When KMS is configured, secrets are encrypted for both age (developers)
/// and KMS (production). At decrypt time, age is tried first (fast, local),
/// then KMS if no age identity is available.
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Envelope {
    version: String,
    /// Age-encrypted ciphertext (always present)
    pub age: String,
    /// KMS-encrypted ciphertext (present when KMS configured)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms: Option<String>,
    /// KMS provider name ("aws" | "gcp")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

#[allow(dead_code)]
impl Envelope {
    /// Create a new hybrid envelope.
    pub fn new(
        age_ciphertext: String,
        kms_ciphertext: Option<String>,
        provider: Option<&KmsProvider>,
    ) -> Self {
        Self {
            version: ENVELOPE_V2.to_string(),
            age: age_ciphertext,
            kms: kms_ciphertext,
            provider: provider.map(|p| p.name().to_string()),
        }
    }

    /// Serialize the envelope to a JSON string.
    pub fn seal(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| {
            CipherError::EncryptionFailed(format!("failed to serialize envelope: {}", e)).into()
        })
    }

    /// Try to parse a ciphertext string as an envelope.
    ///
    /// Returns `None` if the string is not a JSON envelope (raw age ciphertext).
    pub fn parse(ciphertext: &str) -> Option<Self> {
        let envelope: Self = serde_json::from_str(ciphertext).ok()?;
        if envelope.version == ENVELOPE_V2 {
            Some(envelope)
        } else {
            None
        }
    }

    /// Check if a ciphertext string is an envelope (vs raw age).
    pub fn is_envelope(ciphertext: &str) -> bool {
        ciphertext.starts_with('{') && ciphertext.contains(ENVELOPE_V2)
    }
}

/// Trait for KMS encrypt/decrypt operations.
///
/// Implemented by real providers (AWS, GCP) and mock for testing.
#[allow(dead_code)]
pub trait KmsBackend: std::fmt::Debug {
    fn encrypt(&self, plaintext: &str) -> Result<String>;
    fn decrypt(&self, ciphertext: &str) -> Result<String>;
    fn provider(&self) -> &KmsProvider;
}

/// Mock KMS backend for testing.
///
/// Uses simple hex encoding with prefix — NOT cryptographically secure,
/// just validates the plumbing without external crate deps.
#[cfg(any(test, feature = "test-kms"))]
#[derive(Debug)]
pub struct StubKms;

#[cfg(any(test, feature = "test-kms"))]
impl KmsBackend for StubKms {
    fn encrypt(&self, plaintext: &str) -> Result<String> {
        let hex: String = plaintext.bytes().map(|b| format!("{:02x}", b)).collect();
        Ok(format!("mock-kms:{}", hex))
    }

    fn decrypt(&self, ciphertext: &str) -> Result<String> {
        let hex = ciphertext.strip_prefix("mock-kms:").ok_or_else(|| {
            CipherError::DecryptionFailed("not a mock-kms ciphertext".to_string())
        })?;
        let bytes: std::result::Result<Vec<u8>, _> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
            .collect();
        let bytes =
            bytes.map_err(|e| CipherError::DecryptionFailed(format!("invalid hex: {}", e)))?;
        String::from_utf8(bytes)
            .map_err(|e| CipherError::DecryptionFailed(format!("invalid utf8: {}", e)).into())
    }

    fn provider(&self) -> &KmsProvider {
        &KmsProvider::Aws
    }
}

#[cfg(any(test, feature = "test-kms"))]
mod tests {
    #[allow(unused_imports)]
    use super::{Envelope, KmsProvider};

    #[test]
    fn test_detect_aws_arn() {
        let key = "arn:aws:kms:us-east-1:123456789012:key/abc-123";
        assert_eq!(KmsProvider::detect(key), Some(KmsProvider::Aws));
    }

    #[test]
    fn test_detect_aws_alias() {
        let key = "arn:aws:kms:eu-west-1:999:alias/my-key";
        assert_eq!(KmsProvider::detect(key), Some(KmsProvider::Aws));
    }

    #[test]
    fn test_detect_gcp_resource() {
        let key = "projects/my-project/locations/global/keyRings/my-ring/cryptoKeys/my-key";
        assert_eq!(KmsProvider::detect(key), Some(KmsProvider::Gcp));
    }

    #[test]
    fn test_detect_invalid() {
        assert_eq!(KmsProvider::detect("not-a-kms-key"), None);
        assert_eq!(KmsProvider::detect(""), None);
        assert_eq!(KmsProvider::detect("arn:aws:s3:::bucket"), None);
        assert_eq!(KmsProvider::detect("projects/foo"), None);
    }

    #[test]
    fn test_envelope_roundtrip() {
        let envelope = Envelope::new(
            "age-ciphertext-here".to_string(),
            Some("kms-ciphertext-here".to_string()),
            Some(&KmsProvider::Aws),
        );
        let sealed = envelope.seal().unwrap();
        let parsed = Envelope::parse(&sealed).unwrap();
        assert_eq!(parsed.age, "age-ciphertext-here");
        assert_eq!(parsed.kms.unwrap(), "kms-ciphertext-here");
        assert_eq!(parsed.provider.unwrap(), "aws");
    }

    #[test]
    fn test_envelope_age_only() {
        let envelope = Envelope::new("age-ciphertext".to_string(), None, None);
        let sealed = envelope.seal().unwrap();
        let parsed = Envelope::parse(&sealed).unwrap();
        assert_eq!(parsed.age, "age-ciphertext");
        assert!(parsed.kms.is_none());
        assert!(parsed.provider.is_none());
    }

    #[test]
    fn test_envelope_parse_raw_age_returns_none() {
        let raw = "-----BEGIN AGE ENCRYPTED FILE-----\ntest\n-----END AGE ENCRYPTED FILE-----";
        assert!(Envelope::parse(raw).is_none());
    }

    #[test]
    fn test_envelope_is_envelope() {
        let envelope = Envelope::new("age".to_string(), None, None);
        let sealed = envelope.seal().unwrap();
        assert!(Envelope::is_envelope(&sealed));
        assert!(!Envelope::is_envelope("raw age ciphertext"));
    }

    #[test]
    fn test_mock_kms_roundtrip() {
        let mock = StubKms;
        let encrypted = mock.encrypt("secret-value").unwrap();
        let decrypted = mock.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, "secret-value");
    }

    #[test]
    fn test_mock_kms_invalid_ciphertext() {
        let mock = StubKms;
        assert!(mock.decrypt("not-valid-base64!!!").is_err());
    }
}
