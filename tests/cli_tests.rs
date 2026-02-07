//! End-to-end integration tests for the burrow CLI.
//!
//! These tests run the actual compiled binary with a clean environment for each test.

mod harness;

use harness::TestEnv;
use std::fs;

#[test]
fn test_init_creates_config_and_key() {
    let env = TestEnv::new();

    let output = env.init("test-user");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("initialized"));

    // Check that .burrow.toml exists
    let config_path = env.dir.path().join(".burrow.toml");
    assert!(config_path.exists(), ".burrow.toml should exist");

    // Check that a key was created in ~/.burrow/keys/<project_id>/identity.key
    // project_id is derived from the current directory name
    let project_id = env.dir.path().file_name().unwrap().to_string_lossy();
    let identity_path = env
        .home
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
    let env = TestEnv::new();

    // First init should succeed
    let output = env.init("test-user");
    assert!(output.status.success());

    // Second init should fail gracefully
    let output = env
        .cmd()
        .args(["init", "--no-banner", "--name", "test-user"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already initialized"));
}

#[test]
fn test_set_and_get_roundtrip() {
    let env = TestEnv::new();

    env.init("test-user");

    // Set a secret
    let output = env.set("DATABASE_URL", "postgres://localhost/db");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DATABASE_URL"));

    // Get the secret back
    let output = env.cmd().args(["get", "DATABASE_URL"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("postgres://localhost/db"));
}

#[test]
fn test_set_without_init_fails() {
    let env = TestEnv::new();

    let output = env.set("KEY", "VALUE");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not initialized"));
}

#[test]
fn test_list_shows_keys() {
    let env = TestEnv::new();

    env.init("test-user");

    // Initially empty
    let output = env.cmd().arg("list").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("no secrets"));

    // Add a few secrets
    env.set("KEY_ONE", "value1");
    env.set("KEY_TWO", "value2");

    // List should show both
    let output = env.cmd().arg("list").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("KEY_ONE"));
    assert!(stdout.contains("KEY_TWO"));
    assert!(stdout.contains("2 secrets"));
}

#[test]
fn test_rm_removes_secret() {
    let env = TestEnv::new();

    env.init("test-user");
    env.set("TEMP_KEY", "temp_value");

    // Remove it
    let output = env.cmd().args(["rm", "TEMP_KEY"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("removed"));

    // Should no longer be accessible
    let output = env.cmd().args(["get", "TEMP_KEY"]).output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn test_set_with_force_overwrites() {
    let env = TestEnv::new();

    env.init("test-user");
    env.set("OVERWRITE_KEY", "original_value");

    // Without --force should fail
    let output = env.set("OVERWRITE_KEY", "new_value");
    assert!(!output.status.success());

    // With --force should succeed
    let output = env
        .cmd()
        .args(["set", "OVERWRITE_KEY", "new_value", "--force"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Verify new value
    let output = env.cmd().args(["get", "OVERWRITE_KEY"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("new_value"));
}

#[test]
fn test_unlock_creates_env_file() {
    let env = TestEnv::new();

    env.init("test-user");
    env.set("TEST_VAR", "test_value");

    let output = env.cmd().args(["secrets", "unlock"]).output().unwrap();
    assert!(output.status.success());

    // Check that .env was created
    let env_path = env.dir.path().join(".env");
    assert!(env_path.exists(), ".env should exist after unlock");

    let env_content = fs::read_to_string(env_path).unwrap();
    assert!(env_content.contains("TEST_VAR=test_value"));
}

#[test]
fn test_run_injects_env_vars() {
    let env = TestEnv::new();

    env.init("test-user");
    env.set("INJECTED_VAR", "injected_value");

    // Run echo with the environment variable
    #[cfg(unix)]
    {
        let output = env
            .cmd()
            .args(["run", "--", "sh", "-c", "echo $INJECTED_VAR"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("injected_value"));
    }

    #[cfg(windows)]
    {
        let output = env
            .cmd()
            .args(["run", "--", "cmd", "/c", "echo %INJECTED_VAR%"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("injected_value"));
    }
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

    let output = env
        .cmd()
        .args(["secrets", "import", "test.env"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Verify both keys were imported
    let output = env.cmd().args(["get", "IMPORT_KEY1"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("import_value1"));

    let output = env.cmd().args(["get", "IMPORT_KEY2"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("import_value2"));
}

#[test]
fn test_export_outputs_env_format() {
    let env = TestEnv::new();

    env.init("test-user");
    env.set("EXPORT_KEY", "export_value");

    let output = env.cmd().args(["secrets", "export"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("EXPORT_KEY=export_value"));
}

#[test]
fn test_team_list_shows_members() {
    let env = TestEnv::new();

    env.init("alice");

    // Initially just the creator
    let output = env.cmd().args(["team", "list"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("alice"));
}

#[test]
fn test_completions_bash_outputs_script() {
    let env = TestEnv::new();

    let output = env.cmd().args(["completions", "bash"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("_burrow"));
    assert!(stdout.contains("complete"));
}

#[test]
fn test_invalid_key_names_rejected() {
    let env = TestEnv::new();

    env.init("test-user");

    // Keys starting with numbers should fail
    let output = env.set("123BAD", "value");
    assert!(!output.status.success());

    // Empty key should fail
    let output = env.set("", "value");
    assert!(!output.status.success());

    // Keys with special chars should fail
    let output = env.set("KEY-WITH-DASH", "value");
    assert!(!output.status.success());

    let output = env.set("KEY.WITH.DOT", "value");
    assert!(!output.status.success());
}

#[test]
fn test_list_json_output() {
    let env = TestEnv::new();

    env.init("test-user");
    env.set("KEY_JSON", "value_json");

    let output = env.cmd().args(["list", "--json"]).output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");

    // Should have keys array
    assert!(parsed.get("keys").is_some());
}

#[test]
fn test_team_list_json_output() {
    let env = TestEnv::new();

    env.init("alice");

    let output = env.cmd().args(["team", "list", "--json"]).output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");

    // Should have members array
    assert!(parsed.get("members").is_some());
}
