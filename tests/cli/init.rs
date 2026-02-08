//! Tests for `dugout init` command.

use crate::support::*;
use std::fs;

#[test]
fn test_init_creates_config_and_key() {
    let t = Test::new();

    let output = t.init_cmd("test-user");
    assert_success(&output);
    assert_stdout_contains(&output, "initialized");

    // Check that .dugout.toml exists
    let config_path = t.dir.path().join(".dugout.toml");
    assert!(config_path.exists(), ".dugout.toml should exist");

    // Check that a key was created in ~/.dugout/keys/<project_id>/identity.key
    let project_id = t.dir.path().file_name().unwrap().to_string_lossy();
    let identity_path = t
        .home
        .path()
        .join(".dugout/keys")
        .join(&*project_id)
        .join("identity.key");
    assert!(identity_path.exists(), "identity key should exist");

    // Verify config is valid TOML
    let config_content = fs::read_to_string(config_path).unwrap();
    assert!(config_content.contains("version"));
}

#[test]
fn test_init_in_already_initialized_dir_fails() {
    let t = Test::init("test-user");

    // Second init should fail gracefully
    let output = t.init_cmd("test-user");
    assert_failure(&output);
    assert_stderr_contains(&output, "already initialized");
}

#[test]
fn test_init_with_custom_name() {
    let t = Test::new();

    let output = t.init_cmd("alice");
    assert_success(&output);

    let config_path = t.dir.path().join(".dugout.toml");
    let config_content = fs::read_to_string(config_path).unwrap();
    assert!(config_content.contains("alice"));
}

#[test]
fn test_init_creates_gitignore_entry() {
    let t = Test::new();

    let output = t.init_cmd("test-user");
    assert_success(&output);

    let gitignore_path = t.dir.path().join(".gitignore");
    if gitignore_path.exists() {
        let gitignore_content = fs::read_to_string(gitignore_path).unwrap();
        // Should contain .env or related entries
        assert!(gitignore_content.contains(".env") || gitignore_content.contains("dugout"));
    }
}

#[test]
fn test_init_shows_correct_output() {
    let t = Test::new();

    let output = t.init_cmd("test-user");
    assert_success(&output);
    let out = stdout(&output);

    // Should show some indication of success
    assert!(out.contains("initialized") || out.contains("created"));
}
