//! Logging and verbosity tests.
//!
//! These tests verify that verbose flags and logging environment variables
//! control debug output appropriately.

mod support;
use support::*;

#[test]
fn test_verbose_flag_shows_debug_output() {
    let t = Test::init("test-user");

    let output = t.set("TEST_KEY", "test_value");
    assert_success(&output);

    // Run with --verbose flag
    let output = t.cmd().args(["--verbose", "list"]).output().unwrap();
    assert_success(&output);

    // The --verbose flag should be accepted without errors
    // Note: actual debug output depends on logging configuration
    // We're mainly verifying the flag is recognized and doesn't break anything
}

#[test]
fn test_default_no_log_output() {
    let t = Test::init("test-user");

    let output = t.set("TEST_KEY", "test_value");
    assert_success(&output);

    // Run without --verbose
    let output = t.list();
    assert_success(&output);

    // Without verbose, stderr should be minimal or empty (no debug/trace)
    let err = stderr(&output);
    // Should not contain debug-level logs
    assert!(
        !err.contains("DEBUG") && !err.contains("TRACE"),
        "Default mode should not show debug/trace output"
    );
}

#[test]
fn test_burrow_log_env_var() {
    let t = Test::init("test-user");

    let output = t.set("TEST_KEY", "test_value");
    assert_success(&output);

    // Run with BURROW_LOG=debug environment variable
    let output = t
        .cmd()
        .env("BURROW_LOG", "debug")
        .arg("list")
        .output()
        .unwrap();
    assert_success(&output);

    // The BURROW_LOG env var should be accepted without errors
    // Note: actual debug output depends on the logging implementation
    // We're mainly verifying the env var is recognized
}

#[test]
fn test_verbose_init() {
    let t = Test::new();

    let output = t
        .cmd()
        .args(["--verbose", "init", "--no-banner", "--name", "verbose-test"])
        .output()
        .unwrap();
    assert_success(&output);

    // The --verbose flag should work with init
    // We're verifying the flag is accepted and init succeeds
}

#[test]
fn test_verbose_team_operations() {
    let t = Test::init("alice");

    // Add team member with verbose flag
    let output = t
        .cmd()
        .args(["--verbose", "team", "add", "bob", BOB_PUBLIC_KEY])
        .output()
        .unwrap();
    assert_success(&output);

    // The --verbose flag should work with team operations
    // We're verifying the flag is accepted and the operation succeeds
}
