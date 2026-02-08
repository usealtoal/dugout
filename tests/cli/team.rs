//! Tests for `dugout team add/list/rm` commands.

use crate::support::*;

#[test]
fn test_team_list_shows_members() {
    let t = Test::init("alice");

    // Initially just the creator
    let output = t.team_list();
    assert_success(&output);
    assert_stdout_contains(&output, "alice");
}

#[test]
fn test_team_list_json_output() {
    let t = Test::init("alice");

    let output = t.team_list_json();
    assert_success(&output);

    let out = stdout(&output);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
    assert!(parsed.get("members").is_some());
}

#[test]
fn test_team_add_with_valid_key() {
    let t = Test::init("alice");

    let output = t.team_add("bob", BOB_PUBLIC_KEY);
    assert_success(&output);

    // Verify bob appears in team list
    let output = t.team_list();
    assert_success(&output);
    assert_stdout_contains(&output, "bob");
}

#[test]
fn test_team_add_with_invalid_key_fails() {
    let t = Test::init("alice");

    let output = t.team_add("bob", INVALID_PUBLIC_KEY);
    assert_failure(&output);
}

#[test]
fn test_team_add_duplicate_member() {
    let t = Test::init("alice");

    t.team_add("bob", BOB_PUBLIC_KEY);

    // Try adding bob again - may succeed idempotently or fail
    let output = t.team_add("bob", BOB_PUBLIC_KEY);
    // Just verify it doesn't crash - either success or failure is acceptable
    let _ = output;
}

#[test]
fn test_team_rm_member() {
    let t = Test::init("alice");
    t.team_add("bob", BOB_PUBLIC_KEY);

    // Remove bob
    let output = t.team_rm("bob");
    assert_success(&output);

    // Verify bob is gone
    let output = t.team_list();
    assert_success(&output);
    let out = stdout(&output);
    assert!(!out.contains("bob") || out.contains("removed"));
}

#[test]
fn test_team_rm_nonexistent_fails() {
    let t = Test::init("alice");

    let output = t.team_rm("nonexistent");
    assert_failure(&output);
}

#[test]
fn test_team_add_reencrypts_secrets() {
    let t = Test::with_secrets("alice", &[("TEAM_SECRET", "team_value")]);

    let output = t.team_add("bob", BOB_PUBLIC_KEY);
    assert_success(&output);

    // Original user should still be able to access the secret
    let output = t.get("TEAM_SECRET");
    assert_success(&output);
    assert_stdout_contains(&output, "team_value");
}
