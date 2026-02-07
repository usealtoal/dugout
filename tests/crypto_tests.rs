//! Tests for cryptographic operations.

use burrow::core::cipher;

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let identity = age::x25519::Identity::generate();
    let recipient = identity.to_public();

    let plaintext = "super secret password 123!";
    let encrypted = cipher::encrypt(plaintext, &[recipient]).unwrap();

    // Should be ASCII armor format
    assert!(encrypted.contains("-----BEGIN AGE ENCRYPTED FILE-----"));

    let decrypted = cipher::decrypt(&encrypted, &identity).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_encrypt_decrypt_multiple_recipients() {
    let identity1 = age::x25519::Identity::generate();
    let identity2 = age::x25519::Identity::generate();
    let recipient1 = identity1.to_public();
    let recipient2 = identity2.to_public();

    let plaintext = "shared secret";
    let encrypted = cipher::encrypt(plaintext, &[recipient1, recipient2]).unwrap();

    // Both identities should be able to decrypt
    let decrypted1 = cipher::decrypt(&encrypted, &identity1).unwrap();
    let decrypted2 = cipher::decrypt(&encrypted, &identity2).unwrap();

    assert_eq!(decrypted1, plaintext);
    assert_eq!(decrypted2, plaintext);
}

#[test]
fn test_decrypt_with_wrong_key_fails() {
    let identity1 = age::x25519::Identity::generate();
    let identity2 = age::x25519::Identity::generate();
    let recipient1 = identity1.to_public();

    let plaintext = "secret";
    let encrypted = cipher::encrypt(plaintext, &[recipient1]).unwrap();

    // Should fail with wrong key
    let result = cipher::decrypt(&encrypted, &identity2);
    assert!(result.is_err());
}

#[test]
fn test_parse_valid_recipient() {
    let identity = age::x25519::Identity::generate();
    let public_key = identity.to_public().to_string();

    let parsed = cipher::parse_recipient(&public_key).unwrap();
    assert_eq!(parsed.to_string(), public_key);
}

#[test]
fn test_parse_invalid_recipient() {
    let result = cipher::parse_recipient("not a valid key");
    assert!(result.is_err());
}

#[test]
fn test_encrypt_empty_string() {
    let identity = age::x25519::Identity::generate();
    let recipient = identity.to_public();

    let encrypted = cipher::encrypt("", &[recipient]).unwrap();
    let decrypted = cipher::decrypt(&encrypted, &identity).unwrap();

    assert_eq!(decrypted, "");
}

#[test]
fn test_encrypt_unicode() {
    let identity = age::x25519::Identity::generate();
    let recipient = identity.to_public();

    let plaintext = "🔐 Unicode secrets: 日本語, émojis, and more!";
    let encrypted = cipher::encrypt(plaintext, &[recipient]).unwrap();
    let decrypted = cipher::decrypt(&encrypted, &identity).unwrap();

    assert_eq!(decrypted, plaintext);
}
