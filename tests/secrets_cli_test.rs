//! Tests for `burrow set/get/rm/list` commands.

mod harness;
use harness::{assert_failure, assert_success, stderr, stdout, TestEnv};

#[test]
fn test_set_and_get_roundtrip() {
    let env = TestEnv::new();
    env.init("test-user");

    let output = env.set("DATABASE_URL", "postgres://localhost/db");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("DATABASE_URL"));

    let output = env.get("DATABASE_URL");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("postgres://localhost/db"));
}

#[test]
fn test_set_with_force_overwrites() {
    let env = TestEnv::new();
    env.init("test-user");
    env.set("OVERWRITE_KEY", "original_value");

    // Without --force should fail
    let output = env.set("OVERWRITE_KEY", "new_value");
    assert_failure(&output);

    // With --force should succeed
    let output = env.set_force("OVERWRITE_KEY", "new_value");
    assert_success(&output);

    // Verify new value
    let output = env.get("OVERWRITE_KEY");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("new_value"));
}

#[test]
fn test_set_without_init_fails() {
    let env = TestEnv::new();

    let output = env.set("KEY", "VALUE");
    assert_failure(&output);
    let err = stderr(&output);
    assert!(err.contains("not initialized"));
}

#[test]
fn test_invalid_key_names_rejected() {
    let env = TestEnv::new();
    env.init("test-user");

    // Keys starting with numbers should fail
    let output = env.set("123BAD", "value");
    assert_failure(&output);

    // Empty key should fail
    let output = env.set("", "value");
    assert_failure(&output);

    // Keys with special chars should fail
    let output = env.set("KEY-WITH-DASH", "value");
    assert_failure(&output);

    let output = env.set("KEY.WITH.DOT", "value");
    assert_failure(&output);
}

#[test]
fn test_get_nonexistent_key_fails() {
    let env = TestEnv::new();
    env.init("test-user");

    let output = env.get("NONEXISTENT_KEY");
    assert_failure(&output);
}

#[test]
fn test_rm_removes_secret() {
    let env = TestEnv::new();
    env.init("test-user");
    env.set("TEMP_KEY", "temp_value");

    let output = env.rm("TEMP_KEY");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("removed"));

    // Should no longer be accessible
    let output = env.get("TEMP_KEY");
    assert_failure(&output);
}

#[test]
fn test_rm_nonexistent_key_fails() {
    let env = TestEnv::new();
    env.init("test-user");

    let output = env.rm("NONEXISTENT_KEY");
    assert_failure(&output);
}

#[test]
fn test_list_shows_keys() {
    let env = TestEnv::new();
    env.init("test-user");

    // Initially empty
    let output = env.list();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("no secrets") || out.contains("0"));

    // Add a few secrets
    env.set("KEY_ONE", "value1");
    env.set("KEY_TWO", "value2");

    // List should show both
    let output = env.list();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("KEY_ONE"));
    assert!(out.contains("KEY_TWO"));
    assert!(out.contains("2") || out.contains("secrets"));
}

#[test]
fn test_list_json_output() {
    let env = TestEnv::new();
    env.init("test-user");
    env.set("KEY_JSON", "value_json");

    let output = env.list_json();
    assert_success(&output);

    let out = stdout(&output);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
    assert!(parsed.get("keys").is_some());
}

#[test]
fn test_list_empty_vault() {
    let env = TestEnv::new();
    env.init("test-user");

    let output = env.list();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("no secrets") || out.contains("0"));
}

#[test]
fn test_set_multiple_secrets() {
    let env = TestEnv::new();
    env.init("test-user");

    let pairs = [("KEY1", "value1"), ("KEY2", "value2"), ("KEY3", "value3")];
    env.set_multiple(&pairs);

    // Verify all were set
    for (key, expected_val) in &pairs {
        let output = env.get(key);
        assert_success(&output);
        let out = stdout(&output);
        assert!(out.contains(expected_val));
    }
}

#[test]
fn test_get_outputs_raw_value() {
    let env = TestEnv::new();
    env.init("test-user");
    env.set("RAW_KEY", "raw_value");

    let output = env.get("RAW_KEY");
    assert_success(&output);
    let out = stdout(&output);
    // Should contain the value (potentially for piping)
    assert!(out.contains("raw_value"));
}

#[test]
fn test_set_with_special_characters_in_value() {
    let env = TestEnv::new();
    env.init("test-user");

    let special_value = "value!@#$%^&*()_+-=[]{}|;':,.<>?";
    env.set("SPECIAL_KEY", special_value);

    let output = env.get("SPECIAL_KEY");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains(special_value));
}

#[test]
fn test_set_with_multiline_value() {
    let env = TestEnv::new();
    env.init("test-user");

    let multiline = "line1\nline2\nline3";
    env.set("MULTILINE_KEY", multiline);

    let output = env.get("MULTILINE_KEY");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("line1"));
    assert!(out.contains("line2"));
    assert!(out.contains("line3"));
}
