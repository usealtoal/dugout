//! Tests for `burrow team add/list/rm` commands.

mod harness;
use harness::{assert_failure, assert_success, stderr, stdout, TestEnv};

#[test]
fn test_team_list_shows_members() {
    let env = TestEnv::new();
    env.init("alice");

    // Initially just the creator
    let output = env.team_list();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("alice"));
}

#[test]
fn test_team_list_json_output() {
    let env = TestEnv::new();
    env.init("alice");

    let output = env.team_list_json();
    assert_success(&output);

    let out = stdout(&output);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
    assert!(parsed.get("members").is_some());
}

#[test]
fn test_team_add_with_valid_key() {
    let env = TestEnv::new();
    env.init("alice");

    // Generate a valid age public key for testing
    // This is a valid age public key format
    let valid_key = "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p";

    let output = env.team_add("bob", valid_key);
    assert_success(&output);

    // Verify bob appears in team list
    let output = env.team_list();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("bob"));
}

#[test]
fn test_team_add_with_invalid_key_fails() {
    let env = TestEnv::new();
    env.init("alice");

    let invalid_key = "not-a-valid-age-key";

    let output = env.team_add("bob", invalid_key);
    assert_failure(&output);
}

#[test]
fn test_team_rm_member() {
    let env = TestEnv::new();
    env.init("alice");

    let valid_key = "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p";
    env.team_add("bob", valid_key);

    // Remove bob
    let output = env.team_rm("bob");
    assert_success(&output);

    // Verify bob is gone
    let output = env.team_list();
    assert_success(&output);
    let out = stdout(&output);
    assert!(!out.contains("bob") || out.contains("removed"));
}

#[test]
fn test_team_rm_nonexistent_fails() {
    let env = TestEnv::new();
    env.init("alice");

    let output = env.team_rm("nonexistent");
    assert_failure(&output);
}

#[test]
fn test_team_add_reencrypts_secrets() {
    let env = TestEnv::new();
    env.init("alice");
    env.set("TEAM_SECRET", "team_value");

    let valid_key = "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p";
    let output = env.team_add("bob", valid_key);
    assert_success(&output);

    // Original user should still be able to access the secret
    let output = env.get("TEAM_SECRET");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("team_value"));
}
