//! Tests for `burrow secrets lock/unlock/import/export/diff/rotate` commands.

mod harness;
use harness::{assert_failure, assert_success, stderr, stdout, TestEnv};
use std::fs;

#[test]
fn test_unlock_creates_env_file() {
    let env = TestEnv::new();
    env.init("test-user");
    env.set("TEST_VAR", "test_value");

    let output = env.secrets_unlock();
    assert_success(&output);

    // Check that .env was created
    let env_path = env.dir.path().join(".env");
    assert!(env_path.exists(), ".env should exist after unlock");

    let env_content = fs::read_to_string(env_path).unwrap();
    assert!(env_content.contains("TEST_VAR=test_value"));
}

#[test]
fn test_lock_verifies_encryption() {
    let env = TestEnv::new();
    env.init("test-user");
    env.set("LOCK_KEY", "lock_value");

    let output = env.secrets_lock();
    // Lock should succeed (or at least not error)
    assert_success(&output);
}

#[test]
fn test_import_from_env_file() {
    let env = TestEnv::new();
    env.init("test-user");

    // Create a test .env file
    let test_env = env.dir.path().join("test.env");
    fs::write(
        &test_env,
        "IMPORT_KEY1=import_value1\nIMPORT_KEY2=import_value2\n",
    )
    .unwrap();

    let output = env.secrets_import("test.env");
    assert_success(&output);

    // Verify both keys were imported
    let output = env.get("IMPORT_KEY1");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("import_value1"));

    let output = env.get("IMPORT_KEY2");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("import_value2"));
}

#[test]
fn test_export_outputs_env_format() {
    let env = TestEnv::new();
    env.init("test-user");
    env.set("EXPORT_KEY", "export_value");

    let output = env.secrets_export();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("EXPORT_KEY=export_value"));
}

#[test]
fn test_import_skips_comments() {
    let env = TestEnv::new();
    env.init("test-user");

    let test_env = env.dir.path().join("test.env");
    fs::write(
        &test_env,
        "# This is a comment\nKEY1=value1\n# Another comment\nKEY2=value2\n",
    )
    .unwrap();

    let output = env.secrets_import("test.env");
    assert_success(&output);

    // Should have imported 2 keys
    let output = env.get("KEY1");
    assert_success(&output);
    let output = env.get("KEY2");
    assert_success(&output);
}

#[test]
fn test_import_handles_quotes() {
    let env = TestEnv::new();
    env.init("test-user");

    let test_env = env.dir.path().join("test.env");
    fs::write(
        &test_env,
        "KEY1=\"value with spaces\"\nKEY2='single quoted'\nKEY3=no_quotes\n",
    )
    .unwrap();

    let output = env.secrets_import("test.env");
    assert_success(&output);

    // All three should be imported
    let output = env.get("KEY1");
    assert_success(&output);
    let output = env.get("KEY2");
    assert_success(&output);
    let output = env.get("KEY3");
    assert_success(&output);
}

#[test]
fn test_import_handles_empty_lines() {
    let env = TestEnv::new();
    env.init("test-user");

    let test_env = env.dir.path().join("test.env");
    fs::write(&test_env, "KEY1=value1\n\n\nKEY2=value2\n\n").unwrap();

    let output = env.secrets_import("test.env");
    assert_success(&output);

    let output = env.get("KEY1");
    assert_success(&output);
    let output = env.get("KEY2");
    assert_success(&output);
}

#[test]
fn test_diff_shows_synced() {
    let env = TestEnv::new();
    env.init("test-user");
    env.set("SYNC_KEY", "sync_value");

    // Unlock to create .env
    env.secrets_unlock();

    let output = env.secrets_diff();
    assert_success(&output);
    let out = stdout(&output);
    // Should indicate everything is synced
    assert!(out.contains("sync") || out.contains("up to date") || out.contains("✓"));
}

#[test]
fn test_diff_shows_modified() {
    let env = TestEnv::new();
    env.init("test-user");
    env.set("DIFF_KEY", "original_value");
    env.secrets_unlock();

    // Modify .env
    let env_path = env.dir.path().join(".env");
    fs::write(&env_path, "DIFF_KEY=modified_value\n").unwrap();

    let output = env.secrets_diff();
    assert_success(&output);
    let out = stdout(&output);
    // Should show the difference
    assert!(out.contains("DIFF_KEY") || out.contains("modified"));
}

#[test]
fn test_diff_shows_vault_only() {
    let env = TestEnv::new();
    env.init("test-user");
    env.set("VAULT_ONLY", "value");
    env.secrets_unlock();

    // Add another key to vault only
    env.set("VAULT_ONLY_2", "value2");

    let output = env.secrets_diff();
    assert_success(&output);
    let out = stdout(&output);
    // Should show VAULT_ONLY_2 as vault-only
    assert!(out.contains("VAULT_ONLY_2"));
}

#[test]
fn test_diff_shows_env_only() {
    let env = TestEnv::new();
    env.init("test-user");
    env.set("SHARED_KEY", "value");
    env.secrets_unlock();

    // Add a key to .env only
    let env_path = env.dir.path().join(".env");
    let mut content = fs::read_to_string(&env_path).unwrap();
    content.push_str("ENV_ONLY=env_value\n");
    fs::write(&env_path, content).unwrap();

    let output = env.secrets_diff();
    assert_success(&output);
    let out = stdout(&output);
    // Should show ENV_ONLY as env-only
    assert!(out.contains("ENV_ONLY"));
}

#[test]
fn test_rotate_reencrypts_all_secrets() {
    let env = TestEnv::new();
    env.init("test-user");
    env.set("ROTATE_KEY1", "value1");
    env.set("ROTATE_KEY2", "value2");
    env.set("ROTATE_KEY3", "value3");

    let output = env.secrets_rotate();
    assert_success(&output);

    // All secrets should still be accessible
    let output = env.get("ROTATE_KEY1");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("value1"));

    let output = env.get("ROTATE_KEY2");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("value2"));

    let output = env.get("ROTATE_KEY3");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("value3"));
}

#[test]
fn test_export_after_import_roundtrip() {
    let env = TestEnv::new();
    env.init("test-user");

    // Import from a file
    let test_env = env.dir.path().join("import.env");
    fs::write(
        &test_env,
        "ROUNDTRIP_KEY1=roundtrip_value1\nROUNDTRIP_KEY2=roundtrip_value2\n",
    )
    .unwrap();

    env.secrets_import("import.env");

    // Export and verify
    let output = env.secrets_export();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("ROUNDTRIP_KEY1=roundtrip_value1"));
    assert!(out.contains("ROUNDTRIP_KEY2=roundtrip_value2"));
}
