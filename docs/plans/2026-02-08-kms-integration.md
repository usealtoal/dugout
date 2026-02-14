# KMS Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable hybrid encryption where secrets are encrypted for both age keys (developers) and cloud KMS (production servers), with auto-detection of the decrypt path at runtime.

**Architecture:** Extend the existing CipherBackend to support hybrid mode. When a KMS key is configured, every secret gets an envelope containing both an age-encrypted copy and a KMS-encrypted copy. At decrypt time, dugout tries the age path first (fast, local), then falls back to KMS if no age identity is available. The KMS provider is auto-detected from the key format.

**Tech Stack:** Rust, age (crypto), aws-sdk-kms (feature-gated), gcloud CLI (feature-gated), serde_json (envelope format)

---

## Current State

- `CipherBackend::Age` — works, production-ready
- `CipherBackend::AwsKms` — encrypt/decrypt implemented, feature-gated, untested against real AWS
- `CipherBackend::GcpKms` — encrypt/decrypt via gcloud CLI, feature-gated, untested
- `wrap_for_recipients` / `unwrap_for_identity` — envelope wrapping exists but only wraps KMS ciphertext inside age
- `dugout init --cipher aws-kms --kms-key ARN` — CLI flags exist but force KMS-only mode
- No hybrid mode: you're either age OR KMS, not both

## Design

### Hybrid Envelope Format

```json
{
  "version": "dugout-envelope-v2",
  "age": "<age-encrypted ciphertext>",
  "kms": "<base64-encoded KMS ciphertext>",
  "provider": "aws"
}
```

- `age`: always present when recipients exist (developers can always decrypt locally)
- `kms`: present only when KMS is configured
- `provider`: "aws" | "gcp" (tells decrypt which SDK to use)
- Backward compatible: v1 envelopes and raw age ciphertext still decrypt fine

### Decrypt Priority

1. Try age path (if age identity available) — fast, offline, no network
2. Try KMS path (if KMS configured and no age identity) — requires cloud creds
3. Error with guidance

### Config Changes

```toml
[dugout]
version = "0.1.6"
# cipher field is now ONLY for legacy single-backend mode
# new: kms section for hybrid

[kms]
# Auto-detected from format:
# arn:aws:kms:... → AWS
# projects/... → GCP
key = "arn:aws:kms:us-east-1:123456789012:key/abc-123"

[recipients]
alice = "age1..."
bob = "age1..."

[secrets]
API_KEY = '{"version":"dugout-envelope-v2","age":"...","kms":"...","provider":"aws"}'
```

### CLI Changes

```bash
# New: init with hybrid KMS
dugout init --kms-key arn:aws:kms:...
# → sets cipher=age (still default), adds [kms] section

# Existing --cipher flag still works for KMS-only mode (backward compat)
dugout init --cipher aws-kms --kms-key arn:aws:kms:...

# Add/change KMS key on existing vault
dugout config set kms.key "arn:aws:kms:..."

# Re-encrypt all secrets with new envelope format
dugout secrets rotate
```

### Auto-detect Provider

```rust
fn detect_provider(key: &str) -> Option<KmsProvider> {
    if key.starts_with("arn:aws:kms:") { return Some(KmsProvider::Aws); }
    if key.starts_with("projects/") { return Some(KmsProvider::Gcp); }
    None
}
```

---

## Tasks

### Task 1: Envelope v2 format + serialization

**Files:**
- Modify: `src/core/cipher/backend.rs` — add EnvelopeV2 struct, serialize/deserialize
- Test: unit tests in same file

Add the v2 envelope type alongside the existing v1:

```rust
#[derive(Debug, Serialize, Deserialize)]
struct EnvelopeV2 {
    version: String,     // "dugout-envelope-v2"
    age: String,         // age-encrypted ciphertext
    kms: Option<String>, // KMS-encrypted ciphertext (base64)
    provider: Option<String>, // "aws" | "gcp"
}
```

Implement `EnvelopeV2::wrap()` and `EnvelopeV2::unwrap()` methods.
Ensure backward compat: `unwrap` tries v2 first, then v1, then raw age.

Tests:
- Roundtrip v2 envelope
- v1 envelopes still unwrap
- Raw age ciphertext still unwraps
- Missing kms field is fine (age-only mode)

### Task 2: KmsProvider enum + auto-detection

**Files:**
- Create: `src/core/cipher/kms.rs` — shared KMS types and detection
- Modify: `src/core/cipher/mod.rs` — re-export

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum KmsProvider {
    Aws,
    Gcp,
}

impl KmsProvider {
    pub fn detect(key: &str) -> Option<Self> { ... }
    pub fn name(&self) -> &'static str { ... }
}
```

Tests:
- AWS ARN detection
- GCP resource name detection
- Invalid strings return None
- Edge cases (partial matches, empty strings)

### Task 3: Config [kms] section

**Files:**
- Modify: `src/core/config.rs` — add KmsConfig struct and [kms] section
- Modify: `src/core/cipher/backend.rs` — read KMS config for hybrid mode

```rust
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct KmsConfig {
    pub key: Option<String>,
}
```

Config struct gets `pub kms: Option<KmsConfig>` field.
CipherBackend::from_config checks for [kms] section → hybrid mode.

Tests:
- Config with [kms] section loads correctly
- Config without [kms] still works (backward compat)
- Invalid KMS key format rejected

### Task 4: Hybrid encrypt in CipherBackend

**Files:**
- Modify: `src/core/cipher/backend.rs` — hybrid encrypt path

When KMS is configured:
1. Encrypt plaintext with age (for recipients)
2. Encrypt plaintext with KMS
3. Bundle into EnvelopeV2

When KMS is NOT configured:
- Encrypt with age only (existing behavior, no envelope)

Tests:
- Hybrid encrypt produces valid envelope
- Age-only encrypt produces raw age ciphertext (no envelope)
- Envelope contains both age and kms fields

### Task 5: Hybrid decrypt in CipherBackend

**Files:**
- Modify: `src/core/cipher/backend.rs` — hybrid decrypt path

Decrypt logic:
1. If age identity available → try age path from envelope
2. If no age identity + KMS configured → try KMS path from envelope
3. If raw age ciphertext (no envelope) → decrypt with age directly

Tests:
- Decrypt envelope via age path
- Decrypt envelope via KMS path (mocked)
- Decrypt raw age ciphertext (backward compat)
- Decrypt v1 envelope (backward compat)

### Task 6: CLI init --kms-key for hybrid mode

**Files:**
- Modify: `src/cli/init.rs` — handle --kms-key without --cipher
- Modify: `src/core/vault.rs` — Vault::init accepts KmsConfig

When `--kms-key` is provided without `--cipher`:
- Default cipher stays "age"
- Add [kms] section to config
- All secrets get hybrid envelopes

Tests:
- `dugout init --kms-key arn:...` creates config with [kms] section
- Secrets set after init get hybrid envelopes
- Existing `--cipher aws-kms` still works

### Task 7: Mock KMS for testing

**Files:**
- Create: `src/core/cipher/mock_kms.rs` — test-only mock
- Modify: `src/core/cipher/backend.rs` — use mock in tests

The mock encrypts/decrypts using a simple reversible transform (NOT crypto-secure, just for testing the plumbing):

```rust
#[cfg(test)]
pub struct MockKms;

#[cfg(test)]
impl MockKms {
    pub fn encrypt(plaintext: &str) -> String {
        base64::encode(format!("mock-kms:{}", plaintext))
    }
    pub fn decrypt(ciphertext: &str) -> Result<String> {
        let decoded = base64::decode(ciphertext)?;
        let s = String::from_utf8(decoded)?;
        s.strip_prefix("mock-kms:").map(String::from).ok_or(...)
    }
}
```

Tests:
- Mock roundtrip works
- Mock integrated into CipherBackend for hybrid tests

### Task 8: Integration tests

**Files:**
- Create: `tests/kms.rs` — integration tests for hybrid mode

Tests using mock KMS:
- Init with KMS → set secret → get secret (age path)
- Init with KMS → set secret → remove age identity → get secret (KMS path)
- Rotate secrets → envelope format updated
- Mix of old (raw age) and new (envelope) secrets in same vault
- Check status shows KMS info

### Task 9: Documentation

**Files:**
- Create: `KMS.md` — KMS setup guide (root level)
- Modify: `README.md` — update cipher backends section
- Modify: `DEPLOY.md` — add KMS deployment path

KMS.md covers:
- AWS KMS setup (create key, IAM policy, dugout config)
- GCP KMS setup (create keyring, IAM, dugout config)
- Hybrid mode explanation
- Multi-region overview

### Task 10: Clippy, fmt, full test suite, PR

- cargo fmt
- cargo clippy -- -D warnings
- cargo test -- --test-threads=1
- Create PR
