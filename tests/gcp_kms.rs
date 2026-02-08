//! Google Cloud KMS integration tests.
//!
//! These tests require gcloud CLI authentication and a KMS key to run.
//! Set the following environment variables:
//! - Authenticate with `gcloud auth login`
//! - `DUGOUT_TEST_GCP_KEY` (set to a GCP KMS resource name)
//!
//! Example:
//! ```bash
//! gcloud auth login
//! export DUGOUT_TEST_GCP_KEY=projects/my-project/locations/global/keyRings/my-ring/cryptoKeys/my-key
//! cargo test --features gcp,test-gcp,test-kms gcp_kms
//! ```
//!
//! Without credentials, tests will skip gracefully.

#![cfg(feature = "test-gcp")]

mod support;

use crate::support::*;
use dugout::test::{gcp::GcpKms, Cipher};

/// Get the GCP KMS resource name from environment variable.
fn get_gcp_kms_key() -> String {
    std::env::var("DUGOUT_TEST_GCP_KEY").expect("DUGOUT_TEST_GCP_KEY must be set for this test")
}

#[test]
fn test_gcp_kms_encrypt_decrypt_roundtrip() {
    skip_without_gcp!();

    let key = get_gcp_kms_key();
    let cipher = GcpKms::new(key);

    let plaintext = "super-secret-value-12345";
    let ciphertext = cipher.encrypt(plaintext, &[]).expect("failed to encrypt");

    // Ciphertext should be base64-encoded and different from plaintext
    assert_ne!(ciphertext, plaintext);
    assert!(!ciphertext.is_empty());

    // Decrypt should return the original plaintext
    let decrypted = cipher.decrypt(&ciphertext, &()).expect("failed to decrypt");
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_gcp_kms_encrypt_different_values() {
    skip_without_gcp!();

    let key = get_gcp_kms_key();
    let cipher = GcpKms::new(key);

    let values = vec![
        "first-secret",
        "second-secret",
        "third-secret",
        "DATABASE_URL=postgres://localhost/mydb",
        "API_KEY=sk-test-12345",
    ];

    for value in values {
        let ciphertext = cipher.encrypt(value, &[]).expect("failed to encrypt");
        let decrypted = cipher.decrypt(&ciphertext, &()).expect("failed to decrypt");
        assert_eq!(decrypted, value);
    }
}

#[test]
fn test_gcp_kms_large_value() {
    skip_without_gcp!();

    let key = get_gcp_kms_key();
    let cipher = GcpKms::new(key);

    // Create a 10KB payload (10 * 1024 bytes)
    let large_value = "A".repeat(10 * 1024);

    let ciphertext = cipher
        .encrypt(&large_value, &[])
        .expect("failed to encrypt large value");
    let decrypted = cipher
        .decrypt(&ciphertext, &())
        .expect("failed to decrypt large value");

    assert_eq!(decrypted, large_value);
    assert_eq!(decrypted.len(), 10 * 1024);
}

#[test]
fn test_gcp_kms_empty_value() {
    skip_without_gcp!();

    let key = get_gcp_kms_key();
    let cipher = GcpKms::new(key);

    let empty = "";
    let ciphertext = cipher.encrypt(empty, &[]).expect("failed to encrypt");
    let decrypted = cipher.decrypt(&ciphertext, &()).expect("failed to decrypt");

    assert_eq!(decrypted, empty);
    assert!(decrypted.is_empty());
}

#[test]
fn test_gcp_kms_unicode_value() {
    skip_without_gcp!();

    let key = get_gcp_kms_key();
    let cipher = GcpKms::new(key);

    let unicode_values = vec![
        "Hello 世界",
        "🔐 secure secrets 🔒",
        "Émojis: 🚀 🎉 🔥",
        "Ñoño contraseña",
        "עברית",
        "Русский текст",
        "日本語の秘密",
    ];

    for value in unicode_values {
        let ciphertext = cipher.encrypt(value, &[]).expect("failed to encrypt");
        let decrypted = cipher.decrypt(&ciphertext, &()).expect("failed to decrypt");
        assert_eq!(decrypted, value);
    }
}

#[test]
fn test_gcp_kms_name() {
    skip_without_gcp!();

    let key = get_gcp_kms_key();
    let cipher = GcpKms::new(key);

    assert_eq!(cipher.name(), "gcp-kms");
}

#[test]
fn test_hybrid_gcp_cli_init() {
    skip_without_gcp!();

    let key = get_gcp_kms_key();
    let t = Test::new();

    // Initialize with KMS key
    let output = t
        .cmd()
        .args(["init", "--no-banner", "--name", "test-user", "--kms", &key])
        .output()
        .expect("failed to run dugout init --kms");

    assert_success(&output);

    // Verify .dugout.toml contains the KMS configuration
    let config_path = t.dir.path().join(".dugout.toml");
    let config = std::fs::read_to_string(&config_path).expect("failed to read config");
    assert!(config.contains("[kms]"));
    assert!(config.contains(&format!("key = \"{}\"", key)));
}

#[test]
fn test_hybrid_gcp_set_get() {
    skip_without_gcp!();

    let key = get_gcp_kms_key();
    let t = Test::new();

    // Initialize with KMS
    let output = t
        .cmd()
        .args(["init", "--no-banner", "--name", "test-user", "--kms", &key])
        .output()
        .expect("failed to run init");
    assert_success(&output);

    // Set a secret
    let output = t.set("DATABASE_URL", "postgres://localhost/mydb");
    assert_success(&output);

    // Get the secret back
    let output = t.get("DATABASE_URL");
    assert_success(&output);
    assert_stdout_contains(&output, "postgres://localhost/mydb");

    // Set and get multiple secrets
    let output = t.set("API_KEY", "sk-test-12345");
    assert_success(&output);

    let output = t.get("API_KEY");
    assert_success(&output);
    assert_stdout_contains(&output, "sk-test-12345");
}

#[test]
fn test_hybrid_gcp_run() {
    skip_without_gcp!();

    let key = get_gcp_kms_key();
    let t = Test::new();

    // Initialize with KMS
    let output = t
        .cmd()
        .args(["init", "--no-banner", "--name", "test-user", "--kms", &key])
        .output()
        .expect("failed to run init");
    assert_success(&output);

    // Set a secret
    let output = t.set("TEST_SECRET", "injected-via-kms");
    assert_success(&output);

    // Run a command that prints the secret
    #[cfg(unix)]
    {
        let output = t.run(&["sh", "-c", "echo $TEST_SECRET"]);
        assert_success(&output);
        assert_stdout_contains(&output, "injected-via-kms");
    }

    #[cfg(windows)]
    {
        let output = t.run(&["cmd", "/c", "echo %TEST_SECRET%"]);
        assert_success(&output);
        assert_stdout_contains(&output, "injected-via-kms");
    }
}

#[test]
fn test_hybrid_gcp_rotate() {
    skip_without_gcp!();

    let key = get_gcp_kms_key();
    let t = Test::new();

    // Initialize with KMS
    let output = t
        .cmd()
        .args(["init", "--no-banner", "--name", "test-user", "--kms", &key])
        .output()
        .expect("failed to run init");
    assert_success(&output);

    // Set initial secrets
    let output = t.set("SECRET_1", "original-value-1");
    assert_success(&output);
    let output = t.set("SECRET_2", "original-value-2");
    assert_success(&output);

    // Rotate secrets
    let output = t.secrets_rotate();
    assert_success(&output);

    // Verify secrets are still accessible
    let output = t.get("SECRET_1");
    assert_success(&output);
    assert_stdout_contains(&output, "original-value-1");

    let output = t.get("SECRET_2");
    assert_success(&output);
    assert_stdout_contains(&output, "original-value-2");
}
