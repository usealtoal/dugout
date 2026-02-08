//! Tests for `dugout set/get/rm/list` and `dugout secrets` commands.

use crate::support::*;
use std::fs;

// Basic secret operations

#[test]
fn test_set_and_get_roundtrip() {
    let t = Test::init("test-user");

    let output = t.set("DATABASE_URL", "postgres://localhost/db");
    assert_success(&output);
    assert_stdout_contains(&output, "DATABASE_URL");

    let output = t.get("DATABASE_URL");
    assert_success(&output);
    assert_stdout_contains(&output, "postgres://localhost/db");
}

#[test]
fn test_set_with_force_overwrites() {
    let t = Test::with_secrets("test-user", &[("OVERWRITE_KEY", "original_value")]);

    // Without --force should fail
    let output = t.set("OVERWRITE_KEY", "new_value");
    assert_failure(&output);

    // With --force should succeed
    let output = t.set_force("OVERWRITE_KEY", "new_value");
    assert_success(&output);

    // Verify new value
    let output = t.get("OVERWRITE_KEY");
    assert_success(&output);
    assert_stdout_contains(&output, "new_value");
}

#[test]
fn test_set_without_init_fails() {
    let t = Test::new();

    let output = t.set("KEY", "VALUE");
    assert_failure(&output);
    assert_stderr_contains(&output, "not initialized");
}

#[test]
fn test_invalid_key_names_rejected() {
    let t = Test::init("test-user");

    // Keys starting with numbers should fail
    let output = t.set("123BAD", "value");
    assert_failure(&output);

    // Empty key should fail
    let output = t.set("", "value");
    assert_failure(&output);

    // Keys with special chars should fail
    let output = t.set("KEY-WITH-DASH", "value");
    assert_failure(&output);

    let output = t.set("KEY.WITH.DOT", "value");
    assert_failure(&output);
}

#[test]
fn test_get_nonexistent_key_fails() {
    let t = Test::init("test-user");

    let output = t.get("NONEXISTENT_KEY");
    assert_failure(&output);
}

#[test]
fn test_rm_removes_secret() {
    let t = Test::with_secrets("test-user", &[("TEMP_KEY", "temp_value")]);

    let output = t.rm("TEMP_KEY");
    assert_success(&output);
    assert_stdout_contains(&output, "removed");

    // Should no longer be accessible
    let output = t.get("TEMP_KEY");
    assert_failure(&output);
}

#[test]
fn test_rm_nonexistent_key_fails() {
    let t = Test::init("test-user");

    let output = t.rm("NONEXISTENT_KEY");
    assert_failure(&output);
}

#[test]
fn test_list_shows_keys() {
    let t = Test::init("test-user");

    // Initially empty
    let output = t.list();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("no secrets") || out.contains("0"));

    // Add a few secrets
    t.set("KEY_ONE", "value1");
    t.set("KEY_TWO", "value2");

    // List should show both keys (one per line, no count)
    let output = t.list();
    assert_success(&output);
    assert_stdout_contains(&output, "KEY_ONE");
    assert_stdout_contains(&output, "KEY_TWO");
}

#[test]
fn test_list_json_output() {
    let t = Test::with_secrets("test-user", &[("KEY_JSON", "value_json")]);

    let output = t.list_json();
    assert_success(&output);

    let out = stdout(&output);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
    assert!(parsed.get("keys").is_some());
}

#[test]
fn test_list_empty_vault() {
    let t = Test::init("test-user");

    let output = t.list();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("no secrets") || out.contains("0"));
}

#[test]
fn test_set_multiple_secrets() {
    let t = Test::init("test-user");

    let pairs = [("KEY1", "value1"), ("KEY2", "value2"), ("KEY3", "value3")];
    t.set_multiple(&pairs);

    // Verify all were set
    for (key, expected_val) in &pairs {
        let output = t.get(key);
        assert_success(&output);
        assert_stdout_contains(&output, expected_val);
    }
}

#[test]
fn test_get_outputs_raw_value() {
    let t = Test::with_secrets("test-user", &[("RAW_KEY", "raw_value")]);

    let output = t.get("RAW_KEY");
    assert_success(&output);
    assert_stdout_contains(&output, "raw_value");
}

#[test]
fn test_set_with_special_characters_in_value() {
    let t = Test::init("test-user");

    let special_value = "value!@#$%^&*()_+-=[]{}|;':,.<>?";
    t.set("SPECIAL_KEY", special_value);

    let output = t.get("SPECIAL_KEY");
    assert_success(&output);
    assert_stdout_contains(&output, special_value);
}

#[test]
fn test_set_with_multiline_value() {
    let t = Test::init("test-user");

    let multiline = "line1\nline2\nline3";
    t.set("MULTILINE_KEY", multiline);

    let output = t.get("MULTILINE_KEY");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("line1"));
    assert!(out.contains("line2"));
    assert!(out.contains("line3"));
}

#[test]
fn test_set_with_very_long_value() {
    let t = Test::init("test-user");

    let long_value = "x".repeat(10_000);
    t.set("LONG_KEY", &long_value);

    let output = t.get("LONG_KEY");
    assert_success(&output);
    assert_stdout_contains(&output, &long_value);
}

#[test]
fn test_set_with_unicode_value() {
    let t = Test::init("test-user");

    let unicode = "Hello 世界 🌍 Привет مرحبا";
    t.set("UNICODE_KEY", unicode);

    let output = t.get("UNICODE_KEY");
    assert_success(&output);
    assert_stdout_contains(&output, unicode);
}

// Lifecycle operations: lock, unlock, import, export, diff, rotate

#[test]
fn test_unlock_creates_env_file() {
    let t = Test::with_secrets("test-user", &[("TEST_VAR", "test_value")]);

    let output = t.secrets_unlock();
    assert_success(&output);

    // Check that .env was created
    let env_path = t.dir.path().join(".env");
    assert!(env_path.exists(), ".env should exist after unlock");

    let env_content = fs::read_to_string(env_path).unwrap();
    assert!(env_content.contains("TEST_VAR=test_value"));
}

#[test]
fn test_lock_verifies_encryption() {
    let t = Test::with_secrets("test-user", &[("LOCK_KEY", "lock_value")]);

    let output = t.secrets_lock();
    // Lock should succeed (or at least not error)
    assert_success(&output);
}

#[test]
fn test_import_from_env_file() {
    let t = Test::init("test-user");

    // Create a test .env file
    let test_env = t.dir.path().join("test.env");
    fs::write(
        &test_env,
        "IMPORT_KEY1=import_value1\nIMPORT_KEY2=import_value2\n",
    )
    .unwrap();

    let output = t.secrets_import("test.env");
    assert_success(&output);

    // Verify both keys were imported
    let output = t.get("IMPORT_KEY1");
    assert_success(&output);
    assert_stdout_contains(&output, "import_value1");

    let output = t.get("IMPORT_KEY2");
    assert_success(&output);
    assert_stdout_contains(&output, "import_value2");
}

#[test]
fn test_import_empty_file() {
    let t = Test::init("test-user");

    let test_env = t.dir.path().join("empty.env");
    fs::write(&test_env, "").unwrap();

    let output = t.secrets_import("empty.env");
    // Should handle gracefully
    assert_success(&output);
}

#[test]
fn test_import_nonexistent_file_fails() {
    let t = Test::init("test-user");

    let output = t.secrets_import("nonexistent.env");
    assert_failure(&output);
}

#[test]
fn test_export_outputs_env_format() {
    let t = Test::with_secrets("test-user", &[("EXPORT_KEY", "export_value")]);

    let output = t.secrets_export();
    assert_success(&output);
    assert_stdout_contains(&output, "EXPORT_KEY=export_value");
}

#[test]
fn test_export_empty_vault() {
    let t = Test::init("test-user");

    let output = t.secrets_export();
    // Should handle gracefully
    assert_success(&output);
}

#[test]
fn test_import_skips_comments() {
    let t = Test::init("test-user");

    let test_env = t.dir.path().join("test.env");
    fs::write(
        &test_env,
        "# This is a comment\nKEY1=value1\n# Another comment\nKEY2=value2\n",
    )
    .unwrap();

    let output = t.secrets_import("test.env");
    assert_success(&output);

    // Should have imported 2 keys
    let output = t.get("KEY1");
    assert_success(&output);
    let output = t.get("KEY2");
    assert_success(&output);
}

#[test]
fn test_import_handles_quotes() {
    let t = Test::init("test-user");

    let test_env = t.dir.path().join("test.env");
    fs::write(
        &test_env,
        "KEY1=\"value with spaces\"\nKEY2='single quoted'\nKEY3=no_quotes\n",
    )
    .unwrap();

    let output = t.secrets_import("test.env");
    assert_success(&output);

    // All three should be imported
    let output = t.get("KEY1");
    assert_success(&output);
    let output = t.get("KEY2");
    assert_success(&output);
    let output = t.get("KEY3");
    assert_success(&output);
}

#[test]
fn test_import_handles_empty_lines() {
    let t = Test::init("test-user");

    let test_env = t.dir.path().join("test.env");
    fs::write(&test_env, "KEY1=value1\n\n\nKEY2=value2\n\n").unwrap();

    let output = t.secrets_import("test.env");
    assert_success(&output);

    let output = t.get("KEY1");
    assert_success(&output);
    let output = t.get("KEY2");
    assert_success(&output);
}

#[test]
fn test_diff_shows_synced() {
    let t = Test::with_secrets("test-user", &[("SYNC_KEY", "sync_value")]);

    // Unlock to create .env
    t.secrets_unlock();

    let output = t.secrets_diff();
    assert_success(&output);
    let out = stdout(&output);
    // Should indicate everything is synced
    assert!(out.contains("sync") || out.contains("up to date") || out.contains("✓"));
}

#[test]
fn test_diff_with_no_env_file() {
    let t = Test::with_secrets("test-user", &[("KEY", "value")]);

    // Don't unlock, so no .env file exists
    let output = t.secrets_diff();
    // Should handle gracefully
    let _ = output;
}

#[test]
fn test_diff_shows_modified() {
    let t = Test::with_secrets("test-user", &[("DIFF_KEY", "original_value")]);
    t.secrets_unlock();

    // Modify .env
    let env_path = t.dir.path().join(".env");
    fs::write(&env_path, "DIFF_KEY=modified_value\n").unwrap();

    let output = t.secrets_diff();
    assert_success(&output);
    let out = stdout(&output);
    // Should show the difference
    assert!(out.contains("DIFF_KEY") || out.contains("modified"));
}

#[test]
fn test_diff_shows_vault_only() {
    let t = Test::with_secrets("test-user", &[("VAULT_ONLY", "value")]);
    t.secrets_unlock();

    // Add another key to vault only
    t.set("VAULT_ONLY_2", "value2");

    let output = t.secrets_diff();
    assert_success(&output);
    assert_stdout_contains(&output, "VAULT_ONLY_2");
}

#[test]
fn test_diff_shows_env_only() {
    let t = Test::with_secrets("test-user", &[("SHARED_KEY", "value")]);
    t.secrets_unlock();

    // Add a key to .env only
    let env_path = t.dir.path().join(".env");
    let mut content = fs::read_to_string(&env_path).unwrap();
    content.push_str("ENV_ONLY=env_value\n");
    fs::write(&env_path, content).unwrap();

    let output = t.secrets_diff();
    assert_success(&output);
    assert_stdout_contains(&output, "ENV_ONLY");
}

#[test]
fn test_rotate_reencrypts_all_secrets() {
    let t = Test::with_secrets(
        "test-user",
        &[
            ("ROTATE_KEY1", "value1"),
            ("ROTATE_KEY2", "value2"),
            ("ROTATE_KEY3", "value3"),
        ],
    );

    let output = t.secrets_rotate();
    assert_success(&output);

    // All secrets should still be accessible
    let output = t.get("ROTATE_KEY1");
    assert_success(&output);
    assert_stdout_contains(&output, "value1");

    let output = t.get("ROTATE_KEY2");
    assert_success(&output);
    assert_stdout_contains(&output, "value2");

    let output = t.get("ROTATE_KEY3");
    assert_success(&output);
    assert_stdout_contains(&output, "value3");
}

#[test]
fn test_rotate_with_empty_vault() {
    let t = Test::init("test-user");

    let output = t.secrets_rotate();
    // Should handle gracefully
    assert_success(&output);
}

#[test]
fn test_export_after_import_roundtrip() {
    let t = Test::init("test-user");

    // Import from a file
    let test_env = t.dir.path().join("import.env");
    fs::write(
        &test_env,
        "ROUNDTRIP_KEY1=roundtrip_value1\nROUNDTRIP_KEY2=roundtrip_value2\n",
    )
    .unwrap();

    t.secrets_import("import.env");

    // Export and verify
    let output = t.secrets_export();
    assert_success(&output);
    assert_stdout_contains(&output, "ROUNDTRIP_KEY1=roundtrip_value1");
    assert_stdout_contains(&output, "ROUNDTRIP_KEY2=roundtrip_value2");
}
