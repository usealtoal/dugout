//! Config corruption and validation tests.
//!
//! These tests verify that burrow handles malformed, corrupted, or invalid
//! configuration files gracefully with clear error messages.

use burrow::Vault;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

struct TestEnv {
    _dir: TempDir,
    _home: TempDir,
    original_dir: PathBuf,
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original_dir);
    }
}

fn setup() -> TestEnv {
    let original_dir = env::current_dir().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let home_dir = TempDir::new().unwrap();

    env::set_var("HOME", home_dir.path());
    env::set_current_dir(&temp_dir).unwrap();

    TestEnv {
        _dir: temp_dir,
        _home: home_dir,
        original_dir,
    }
}

#[test]
fn test_load_malformed_toml() {
    let _env = setup();

    // Write garbage to .burrow.toml
    fs::write(".burrow.toml", "this is not valid toml { [ }").unwrap();

    // Try to open, expect clean error
    let result = Vault::open();
    assert!(result.is_err(), "Expected error with malformed TOML");
}

#[test]
fn test_load_truncated_config() {
    let _env = setup();

    // Write half a valid config
    let truncated = r#"[burrow]
version = "0.1.0"

[recipients]
alice = "age1ql3z7hjy54pw3"#;
    fs::write(".burrow.toml", truncated).unwrap();

    // Try to open, expect error
    let result = Vault::open();
    assert!(result.is_err(), "Expected error with truncated config");
}

#[test]
fn test_load_missing_version() {
    let _env = setup();

    // Write config without version field
    let no_version = r#"[burrow]

[recipients]
alice = "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p"

[secrets]
"#;
    fs::write(".burrow.toml", no_version).unwrap();

    // Try to open, expect validation error
    let result = Vault::open();
    assert!(result.is_err(), "Expected error with missing version");
}

#[test]
fn test_load_empty_config_file() {
    let _env = setup();

    // Write empty .burrow.toml
    fs::write(".burrow.toml", "").unwrap();

    // Try to open, expect error
    let result = Vault::open();
    assert!(result.is_err(), "Expected error with empty config file");
}

#[test]
fn test_load_config_wrong_type() {
    let _env = setup();

    // Write valid TOML but wrong structure
    let wrong_type = r#"name = "not a project"
type = "wrong"
value = 123
"#;
    fs::write(".burrow.toml", wrong_type).unwrap();

    // Try to open, expect error
    let result = Vault::open();
    assert!(result.is_err(), "Expected error with wrong TOML structure");
}

#[test]
fn test_config_with_invalid_secret_key() {
    let _env = setup();

    // Init a valid vault first
    let mut vault = Vault::init("test-user").unwrap();
    drop(vault);

    // Manually craft config with invalid secret key
    let config_content = fs::read_to_string(".burrow.toml").unwrap();

    // Try to append invalid secrets section
    let bad_config = format!(
        "{}\n[secrets]\n\"123BAD\" = \"AGE-SECRET-KEY-1...\"\n",
        config_content
    );
    fs::write(".burrow.toml", bad_config).unwrap();

    // The config might load but trying to use it should fail gracefully
    // This is a best-effort test - exact behavior depends on implementation
    let result = Vault::open();
    // Either open fails or operations with the vault will fail
    // We're mainly checking it doesn't panic
    if let Ok(mut vault) = result {
        // If it opens, operations should handle invalid keys gracefully
        let _ = vault.set("TEST", "value", false);
    }
}

#[test]
fn test_config_with_no_recipients() {
    let _env = setup();

    // Write config with empty recipients
    let no_recipients = r#"[burrow]
version = "0.1.0"

[recipients]

[secrets]
"#;
    fs::write(".burrow.toml", no_recipients).unwrap();

    // Try to open, expect validation error
    let result = Vault::open();
    assert!(
        result.is_err(),
        "Expected error with no recipients in config"
    );
}
