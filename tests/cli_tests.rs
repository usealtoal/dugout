//! End-to-end integration tests for the burrow CLI.
//!
//! These tests run the actual compiled binary with a clean environment for each test.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to create a fresh burrow command with isolated temp directories.
#[allow(deprecated)]
fn burrow_cmd(tempdir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("burrow").unwrap();
    // Set HOME to tempdir so keys don't pollute real home
    cmd.env("HOME", tempdir.path());
    cmd.current_dir(tempdir.path());
    cmd
}

#[test]
fn test_init_creates_config_and_key() {
    let temp = TempDir::new().unwrap();

    burrow_cmd(&temp)
        .arg("init")
        .arg("--no-banner")
        .arg("--name")
        .arg("test-user")
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized"));

    // Check that .burrow.toml exists
    let config_path = temp.path().join(".burrow.toml");
    assert!(config_path.exists(), ".burrow.toml should exist");

    // Check that a key was created in ~/.burrow/keys/<project_id>/identity.key
    // project_id is derived from the current directory name
    let project_id = temp.path().file_name().unwrap().to_string_lossy();
    let identity_path = temp
        .path()
        .join(".burrow/keys")
        .join(&*project_id)
        .join("identity.key");
    assert!(identity_path.exists(), "identity key should exist");

    // Verify config is valid TOML
    let config_content = fs::read_to_string(config_path).unwrap();
    assert!(config_content.contains("version"));
}

#[test]
fn test_init_in_already_initialized_dir_fails() {
    let temp = TempDir::new().unwrap();

    // First init should succeed
    burrow_cmd(&temp)
        .arg("init")
        .arg("--no-banner")
        .arg("--name")
        .arg("test-user")
        .assert()
        .success();

    // Second init should fail gracefully
    burrow_cmd(&temp)
        .arg("init")
        .arg("--no-banner")
        .arg("--name")
        .arg("test-user")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already initialized"));
}

#[test]
fn test_set_and_get_roundtrip() {
    let temp = TempDir::new().unwrap();

    // Initialize
    burrow_cmd(&temp)
        .arg("init")
        .arg("--no-banner")
        .arg("--name")
        .arg("test-user")
        .assert()
        .success();

    // Set a secret
    burrow_cmd(&temp)
        .arg("set")
        .arg("DATABASE_URL")
        .arg("postgres://localhost/db")
        .assert()
        .success()
        .stdout(predicate::str::contains("DATABASE_URL"));

    // Get the secret back
    burrow_cmd(&temp)
        .arg("get")
        .arg("DATABASE_URL")
        .assert()
        .success()
        .stdout(predicate::str::contains("postgres://localhost/db"));
}

#[test]
fn test_set_without_init_fails() {
    let temp = TempDir::new().unwrap();

    burrow_cmd(&temp)
        .arg("set")
        .arg("KEY")
        .arg("VALUE")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized"));
}

#[test]
fn test_list_shows_keys() {
    let temp = TempDir::new().unwrap();

    burrow_cmd(&temp)
        .arg("init")
        .arg("--no-banner")
        .arg("--name")
        .arg("test-user")
        .assert()
        .success();

    // Initially empty
    burrow_cmd(&temp)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("no secrets"));

    // Add a few secrets
    burrow_cmd(&temp)
        .arg("set")
        .arg("KEY_ONE")
        .arg("value1")
        .assert()
        .success();

    burrow_cmd(&temp)
        .arg("set")
        .arg("KEY_TWO")
        .arg("value2")
        .assert()
        .success();

    // List should show both
    burrow_cmd(&temp)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("KEY_ONE"))
        .stdout(predicate::str::contains("KEY_TWO"))
        .stdout(predicate::str::contains("2 secrets"));
}

#[test]
fn test_rm_removes_secret() {
    let temp = TempDir::new().unwrap();

    burrow_cmd(&temp)
        .arg("init")
        .arg("--no-banner")
        .arg("--name")
        .arg("test-user")
        .assert()
        .success();

    burrow_cmd(&temp)
        .arg("set")
        .arg("TEMP_KEY")
        .arg("temp_value")
        .assert()
        .success();

    // Remove it
    burrow_cmd(&temp)
        .arg("rm")
        .arg("TEMP_KEY")
        .assert()
        .success()
        .stdout(predicate::str::contains("removed"));

    // Should no longer be accessible
    burrow_cmd(&temp)
        .arg("get")
        .arg("TEMP_KEY")
        .assert()
        .failure();
}

#[test]
fn test_set_with_force_overwrites() {
    let temp = TempDir::new().unwrap();

    burrow_cmd(&temp)
        .arg("init")
        .arg("--no-banner")
        .arg("--name")
        .arg("test-user")
        .assert()
        .success();

    burrow_cmd(&temp)
        .arg("set")
        .arg("OVERWRITE_KEY")
        .arg("original_value")
        .assert()
        .success();

    // Without --force should fail
    burrow_cmd(&temp)
        .arg("set")
        .arg("OVERWRITE_KEY")
        .arg("new_value")
        .assert()
        .failure();

    // With --force should succeed
    burrow_cmd(&temp)
        .arg("set")
        .arg("OVERWRITE_KEY")
        .arg("new_value")
        .arg("--force")
        .assert()
        .success();

    // Verify new value
    burrow_cmd(&temp)
        .arg("get")
        .arg("OVERWRITE_KEY")
        .assert()
        .success()
        .stdout(predicate::str::contains("new_value"));
}

#[test]
fn test_unlock_creates_env_file() {
    let temp = TempDir::new().unwrap();

    burrow_cmd(&temp)
        .arg("init")
        .arg("--no-banner")
        .arg("--name")
        .arg("test-user")
        .assert()
        .success();

    burrow_cmd(&temp)
        .arg("set")
        .arg("TEST_VAR")
        .arg("test_value")
        .assert()
        .success();

    burrow_cmd(&temp).arg("unlock").assert().success();

    // Check that .env was created
    let env_path = temp.path().join(".env");
    assert!(env_path.exists(), ".env should exist after unlock");

    let env_content = fs::read_to_string(env_path).unwrap();
    assert!(env_content.contains("TEST_VAR=test_value"));
}

#[test]
fn test_run_injects_env_vars() {
    let temp = TempDir::new().unwrap();

    burrow_cmd(&temp)
        .arg("init")
        .arg("--no-banner")
        .arg("--name")
        .arg("test-user")
        .assert()
        .success();

    burrow_cmd(&temp)
        .arg("set")
        .arg("INJECTED_VAR")
        .arg("injected_value")
        .assert()
        .success();

    // Run echo with the environment variable
    #[cfg(unix)]
    {
        burrow_cmd(&temp)
            .arg("run")
            .arg("--")
            .arg("sh")
            .arg("-c")
            .arg("echo $INJECTED_VAR")
            .assert()
            .success()
            .stdout(predicate::str::contains("injected_value"));
    }

    #[cfg(windows)]
    {
        burrow_cmd(&temp)
            .arg("run")
            .arg("--")
            .arg("cmd")
            .arg("/c")
            .arg("echo %INJECTED_VAR%")
            .assert()
            .success()
            .stdout(predicate::str::contains("injected_value"));
    }
}

#[test]
fn test_import_from_env_file() {
    let temp = TempDir::new().unwrap();

    burrow_cmd(&temp)
        .arg("init")
        .arg("--no-banner")
        .arg("--name")
        .arg("test-user")
        .assert()
        .success();

    // Create a test .env file
    let test_env = temp.path().join("test.env");
    fs::write(
        &test_env,
        "IMPORT_KEY1=import_value1\nIMPORT_KEY2=import_value2\n",
    )
    .unwrap();

    burrow_cmd(&temp)
        .arg("import")
        .arg("test.env")
        .assert()
        .success();

    // Verify both keys were imported
    burrow_cmd(&temp)
        .arg("get")
        .arg("IMPORT_KEY1")
        .assert()
        .success()
        .stdout(predicate::str::contains("import_value1"));

    burrow_cmd(&temp)
        .arg("get")
        .arg("IMPORT_KEY2")
        .assert()
        .success()
        .stdout(predicate::str::contains("import_value2"));
}

#[test]
fn test_export_outputs_env_format() {
    let temp = TempDir::new().unwrap();

    burrow_cmd(&temp)
        .arg("init")
        .arg("--no-banner")
        .arg("--name")
        .arg("test-user")
        .assert()
        .success();

    burrow_cmd(&temp)
        .arg("set")
        .arg("EXPORT_KEY")
        .arg("export_value")
        .assert()
        .success();

    burrow_cmd(&temp)
        .arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("EXPORT_KEY=export_value"));
}

#[test]
fn test_team_list_shows_members() {
    let temp = TempDir::new().unwrap();

    burrow_cmd(&temp)
        .arg("init")
        .arg("--no-banner")
        .arg("--name")
        .arg("alice")
        .assert()
        .success();

    // Initially just the creator
    burrow_cmd(&temp)
        .arg("team")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("alice"));
}

#[test]
fn test_completions_bash_outputs_script() {
    let temp = TempDir::new().unwrap();

    burrow_cmd(&temp)
        .arg("completions")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("_burrow"))
        .stdout(predicate::str::contains("complete"));
}

#[test]
fn test_invalid_key_names_rejected() {
    let temp = TempDir::new().unwrap();

    burrow_cmd(&temp)
        .arg("init")
        .arg("--no-banner")
        .arg("--name")
        .arg("test-user")
        .assert()
        .success();

    // Keys starting with numbers should fail
    burrow_cmd(&temp)
        .arg("set")
        .arg("123BAD")
        .arg("value")
        .assert()
        .failure();

    // Empty key should fail
    burrow_cmd(&temp)
        .arg("set")
        .arg("")
        .arg("value")
        .assert()
        .failure();

    // Keys with special chars should fail
    burrow_cmd(&temp)
        .arg("set")
        .arg("KEY-WITH-DASH")
        .arg("value")
        .assert()
        .failure();

    burrow_cmd(&temp)
        .arg("set")
        .arg("KEY.WITH.DOT")
        .arg("value")
        .assert()
        .failure();
}

#[test]
fn test_list_json_output() {
    let temp = TempDir::new().unwrap();

    burrow_cmd(&temp)
        .arg("init")
        .arg("--no-banner")
        .arg("--name")
        .arg("test-user")
        .assert()
        .success();

    burrow_cmd(&temp)
        .arg("set")
        .arg("KEY_JSON")
        .arg("value_json")
        .assert()
        .success();

    let output = burrow_cmd(&temp)
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");

    // Should have keys array
    assert!(parsed.get("keys").is_some());
}

#[test]
fn test_team_list_json_output() {
    let temp = TempDir::new().unwrap();

    burrow_cmd(&temp)
        .arg("init")
        .arg("--no-banner")
        .arg("--name")
        .arg("alice")
        .assert()
        .success();

    let output = burrow_cmd(&temp)
        .arg("team")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");

    // Should have members array
    assert!(parsed.get("members").is_some());
}
