//! Tests for `dugout setup` and `dugout whoami` commands.

use crate::support::*;
use std::fs;

#[test]
fn test_setup_creates_global_identity() {
    let t = Test::new();

    let output = t.cmd().arg("setup").output().unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "generated identity");

    // Check that ~/.dugout/identity exists
    let identity_path = t.home.path().join(".dugout/identity");
    assert!(identity_path.exists(), "~/.dugout/identity should exist");

    // Check that ~/.dugout/identity.pub exists
    let pubkey_path = t.home.path().join(".dugout/identity.pub");
    assert!(pubkey_path.exists(), "~/.dugout/identity.pub should exist");

    // Verify public key starts with age1
    let pubkey = fs::read_to_string(&pubkey_path).unwrap();
    assert!(pubkey.trim().starts_with("age1"));
}

#[test]
fn test_setup_idempotent_without_force() {
    let t = Test::new();

    // First setup
    let output = t.cmd().arg("setup").output().unwrap();
    assert_success(&output);

    let pubkey_path = t.home.path().join(".dugout/identity.pub");
    let first_pubkey = fs::read_to_string(&pubkey_path).unwrap();

    // Second setup without --force should warn and not overwrite
    let output = t.cmd().arg("setup").output().unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "already exists");

    let second_pubkey = fs::read_to_string(&pubkey_path).unwrap();
    assert_eq!(
        first_pubkey, second_pubkey,
        "public key should not change without --force"
    );
}

#[test]
fn test_setup_with_force_overwrites() {
    let t = Test::new();

    // First setup
    let output = t.cmd().arg("setup").output().unwrap();
    assert_success(&output);

    let pubkey_path = t.home.path().join(".dugout/identity.pub");
    let first_pubkey = fs::read_to_string(&pubkey_path).unwrap();

    // Second setup with --force should overwrite
    let output = t.cmd().args(["setup", "--force"]).output().unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "generated identity");

    let second_pubkey = fs::read_to_string(&pubkey_path).unwrap();
    assert_ne!(
        first_pubkey, second_pubkey,
        "public key should change with --force"
    );
}

#[test]
fn test_whoami_prints_public_key() {
    let t = Test::new();

    // Setup identity first
    let setup_output = t.cmd().arg("setup").output().unwrap();
    assert_success(&setup_output);

    // Run whoami
    let output = t.cmd().arg("whoami").output().unwrap();
    assert_success(&output);

    let output_str = stdout(&output);
    assert!(output_str.trim().starts_with("age1"));

    // Should match the public key file
    let pubkey_path = t.home.path().join(".dugout/identity.pub");
    let expected_pubkey = fs::read_to_string(&pubkey_path).unwrap();
    assert_eq!(output_str.trim(), expected_pubkey.trim());
}

#[test]
fn test_whoami_without_setup_fails() {
    let t = Test::new();

    let output = t.cmd().arg("whoami").output().unwrap();
    assert_failure(&output);
    assert_stderr_contains(&output, "no identity found");
    assert_stdout_contains(&output, "dugout setup");
}

#[test]
fn test_setup_output_includes_paths() {
    let t = Test::new();

    let output = t.cmd().arg("setup").output().unwrap();
    assert_success(&output);

    // Should show success and the public key
    assert_stdout_contains(&output, "generated identity");
    let out = stdout(&output);
    assert!(out.contains("age1")); // public key should be displayed
}
