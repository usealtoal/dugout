//! AWS KMS integration tests.
//!
//! These tests require real AWS credentials and a KMS key to run.
//! Set the following environment variables:
//! - `AWS_ACCESS_KEY_ID` (or use AWS credential chain)
//! - `AWS_SECRET_ACCESS_KEY` (or use AWS credential chain)
//! - `DUGOUT_TEST_KMS_KEY` (set to an AWS KMS key ARN)
//!
//! Example:
//! ```bash
//! export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
//! export AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
//! export DUGOUT_TEST_KMS_KEY=arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012
//! cargo test --features aws,test-aws,test-kms aws_kms
//! ```
//!
//! Without credentials, tests will skip gracefully.

#![cfg(feature = "test-aws")]

mod support;

use crate::support::*;
use dugout::test::{aws::AwsKms, Cipher};

/// Get the AWS KMS key from environment variable.
fn get_aws_kms_key() -> String {
    std::env::var("DUGOUT_TEST_KMS_KEY").expect("DUGOUT_TEST_KMS_KEY must be set for this test")
}

#[test]
fn test_aws_kms_encrypt_decrypt_roundtrip() {
    skip_without_aws!();

    let key = get_aws_kms_key();
    let cipher = AwsKms::new(key);

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
fn test_aws_kms_encrypt_different_values() {
    skip_without_aws!();

    let key = get_aws_kms_key();
    let cipher = AwsKms::new(key);

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
fn test_aws_kms_large_value() {
    skip_without_aws!();

    let key = get_aws_kms_key();
    let cipher = AwsKms::new(key);

    // KMS symmetric encrypt max is 4096 bytes — use 4000 to stay under
    let large_value = "A".repeat(4000);

    let ciphertext = cipher
        .encrypt(&large_value, &[])
        .expect("failed to encrypt large value");
    let decrypted = cipher
        .decrypt(&ciphertext, &())
        .expect("failed to decrypt large value");

    assert_eq!(decrypted, large_value);
    assert_eq!(decrypted.len(), 4000);
}

#[test]
fn test_aws_kms_empty_value() {
    skip_without_aws!();

    let key = get_aws_kms_key();
    let cipher = AwsKms::new(key);

    // AWS KMS requires at least 1 byte — empty string should error
    let result = cipher.encrypt("", &[]);
    assert!(result.is_err(), "KMS should reject empty plaintext");
}

#[test]
fn test_aws_kms_unicode_value() {
    skip_without_aws!();

    let key = get_aws_kms_key();
    let cipher = AwsKms::new(key);

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
fn test_aws_kms_name() {
    skip_without_aws!();

    let key = get_aws_kms_key();
    let cipher = AwsKms::new(key);

    assert_eq!(cipher.name(), "aws-kms");
}

#[test]
fn test_hybrid_aws_cli_init() {
    skip_without_aws!();

    let key = get_aws_kms_key();
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
fn test_hybrid_aws_set_get() {
    skip_without_aws!();

    let key = get_aws_kms_key();
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
fn test_hybrid_aws_run() {
    skip_without_aws!();

    let key = get_aws_kms_key();
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
fn test_hybrid_aws_rotate() {
    skip_without_aws!();

    let key = get_aws_kms_key();
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
