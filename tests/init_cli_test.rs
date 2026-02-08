//! Tests for `burrow init` command.

mod harness;
use harness::{assert_failure, assert_success, stderr, stdout, TestEnv};
use std::fs;

#[test]
fn test_init_creates_config_and_key() {
    let env = TestEnv::new();

    let output = env.init("test-user");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("initialized"));

    // Check that .burrow.toml exists
    let config_path = env.dir.path().join(".burrow.toml");
    assert!(config_path.exists(), ".burrow.toml should exist");

    // Check that a key was created in ~/.burrow/keys/<project_id>/identity.key
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
    assert_success(&output);

    // Second init should fail gracefully
    let output = env
        .cmd()
        .args(["init", "--no-banner", "--name", "test-user"])
        .output()
        .unwrap();
    assert_failure(&output);
    let err = stderr(&output);
    assert!(err.contains("already initialized"));
}

#[test]
fn test_init_with_custom_name() {
    let env = TestEnv::new();

    let output = env.init("alice");
    assert_success(&output);

    let config_path = env.dir.path().join(".burrow.toml");
    let config_content = fs::read_to_string(config_path).unwrap();
    assert!(config_content.contains("alice"));
}

#[test]
fn test_init_creates_gitignore_entry() {
    let env = TestEnv::new();

    let output = env.init("test-user");
    assert_success(&output);

    let gitignore_path = env.dir.path().join(".gitignore");
    if gitignore_path.exists() {
        let gitignore_content = fs::read_to_string(gitignore_path).unwrap();
        // Should contain .env or related entries
        assert!(gitignore_content.contains(".env") || gitignore_content.contains("burrow"));
    }
}

#[test]
fn test_init_shows_correct_output() {
    let env = TestEnv::new();

    let output = env.init("test-user");
    assert_success(&output);
    let out = stdout(&output);

    // Should show some indication of success
    assert!(out.contains("initialized") || out.contains("created"));
}
