//! Domain type tests.
//!
//! These tests verify the domain types work correctly at the API level.
//! Unit tests in src/core/domain/* already cover most of the behavior.

use dugout::Env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_env_load_basic() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.env");
    fs::write(&path, "KEY1=value1\nKEY2=value2\n").unwrap();

    let env = Env::load(&path).unwrap();
    assert_eq!(env.get("KEY1"), Some("value1"));
    assert_eq!(env.get("KEY2"), Some("value2"));
}

#[test]
fn test_env_load_with_quotes() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.env");
    let content = r#"KEY1="quoted value"
KEY2='single quoted'
KEY3=unquoted"#;
    fs::write(&path, content).unwrap();

    let env = Env::load(&path).unwrap();
    assert_eq!(env.get("KEY1"), Some("quoted value"));
    assert_eq!(env.get("KEY2"), Some("single quoted"));
    assert_eq!(env.get("KEY3"), Some("unquoted"));
}

#[test]
fn test_env_load_skips_comments() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.env");
    fs::write(&path, "# This is a comment\nKEY=value\n# Another comment\n").unwrap();

    let env = Env::load(&path).unwrap();
    assert_eq!(env.get("KEY"), Some("value"));
}

#[test]
fn test_env_load_handles_empty_lines() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.env");
    fs::write(&path, "KEY1=value1\n\n\nKEY2=value2\n").unwrap();

    let env = Env::load(&path).unwrap();
    assert_eq!(env.get("KEY1"), Some("value1"));
    assert_eq!(env.get("KEY2"), Some("value2"));
}

#[test]
fn test_env_from_pairs_and_display() {
    let pairs = vec![
        ("KEY1".to_string(), "value1".to_string()),
        ("KEY2".to_string(), "value2".to_string()),
    ];
    let env = Env::from_pairs(pairs, PathBuf::from(".env"));

    let display = format!("{}", env);
    assert!(display.contains("KEY1=value1"));
    assert!(display.contains("KEY2=value2"));
}
