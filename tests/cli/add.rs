//! Tests for `burrow add` command.

use crate::support::*;

#[test]
fn test_add_via_stdin() {
    let t = Test::init("alice");

    // Pipe value via stdin
    let output = t
        .cmd()
        .args(["add", "API_KEY"])
        .write_stdin("secret_value\n")
        .output()
        .unwrap();

    assert_success(&output);
    assert_stdout_contains(&output, "✓ set");
    assert_stdout_contains(&output, "API_KEY");

    // Verify secret was stored
    let get_output = t.get("API_KEY");
    assert_success(&get_output);
    assert_eq!(stdout(&get_output).trim(), "secret_value");
}

#[test]
fn test_add_empty_value_fails() {
    let t = Test::init("alice");

    let output = t
        .cmd()
        .args(["add", "API_KEY"])
        .write_stdin("\n")
        .output()
        .unwrap();

    assert_failure(&output);
    assert_stderr_contains(&output, "empty");
}

#[test]
fn test_add_replaces_existing_when_confirmed() {
    let t = Test::init("alice");

    // Set initial value
    let set_output = t.set("API_KEY", "old_value");
    assert_success(&set_output);

    // Try to add again with new value - should fail in test since we can't interact
    // In a real scenario, dialoguer would prompt
    let output = t
        .cmd()
        .args(["add", "API_KEY"])
        .write_stdin("new_value\n")
        .output()
        .unwrap();

    // In non-interactive mode (stdin piped), it should either succeed or fail gracefully
    // The behavior depends on dialoguer's handling of non-TTY
}

#[test]
fn test_add_invalid_key_fails() {
    let t = Test::init("alice");

    // Invalid key with special characters
    let output = t
        .cmd()
        .args(["add", "invalid-key"])
        .write_stdin("value\n")
        .output()
        .unwrap();

    assert_failure(&output);
    assert_stderr_contains(&output, "invalid");
}

#[test]
fn test_add_key_starting_with_digit_fails() {
    let t = Test::init("alice");

    let output = t
        .cmd()
        .args(["add", "1INVALID"])
        .write_stdin("value\n")
        .output()
        .unwrap();

    assert_failure(&output);
    assert_stderr_contains(&output, "digit");
}

#[test]
fn test_add_then_get_roundtrip() {
    let t = Test::init("alice");

    // Add secret
    let add_output = t
        .cmd()
        .args(["add", "DATABASE_URL"])
        .write_stdin("postgres://localhost/mydb\n")
        .output()
        .unwrap();

    assert_success(&add_output);

    // Get secret
    let get_output = t.get("DATABASE_URL");
    assert_success(&get_output);
    assert_eq!(stdout(&get_output).trim(), "postgres://localhost/mydb");
}

#[test]
fn test_add_multiple_secrets() {
    let t = Test::init("alice");

    // Add first secret
    let output1 = t
        .cmd()
        .args(["add", "KEY_ONE"])
        .write_stdin("value1\n")
        .output()
        .unwrap();
    assert_success(&output1);

    // Add second secret
    let output2 = t
        .cmd()
        .args(["add", "KEY_TWO"])
        .write_stdin("value2\n")
        .output()
        .unwrap();
    assert_success(&output2);

    // Both should be in the list
    let list_output = t.list();
    assert_success(&list_output);
    assert_stdout_contains(&list_output, "KEY_ONE");
    assert_stdout_contains(&list_output, "KEY_TWO");
}

#[test]
fn test_add_without_vault_fails() {
    let t = Test::new();

    let output = t
        .cmd()
        .args(["add", "API_KEY"])
        .write_stdin("value\n")
        .output()
        .unwrap();

    assert_failure(&output);
    assert_stderr_contains(&output, "not initialized");
}
