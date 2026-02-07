//! Tests for configuration management.

use burrow::core::config::BurrowConfig;
use tempfile::TempDir;

#[test]
fn test_config_new() {
    let config = BurrowConfig::new();
    assert_eq!(config.burrow.version, env!("CARGO_PKG_VERSION"));
    assert!(config.recipients.is_empty());
    assert!(config.secrets.is_empty());
}

#[test]
fn test_config_save_and_load() {
    let original_dir = std::env::current_dir().unwrap();
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    let mut config = BurrowConfig::new();
    config.recipients.insert("alice".to_string(), "age1test123".to_string());
    config.secrets.insert("KEY".to_string(), "encrypted_value".to_string());

    // Save
    config.save().unwrap();
    assert!(BurrowConfig::exists());

    // Load
    let loaded = BurrowConfig::load().unwrap();
    assert_eq!(loaded.recipients.len(), 1);
    assert_eq!(loaded.secrets.len(), 1);
    assert_eq!(loaded.recipients.get("alice").unwrap(), "age1test123");
    assert_eq!(loaded.secrets.get("KEY").unwrap(), "encrypted_value");

    // Restore directory
    std::env::set_current_dir(&original_dir).unwrap();
}

#[test]
fn test_config_load_not_initialized() {
    let original_dir = std::env::current_dir().unwrap();
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    let result = BurrowConfig::load();
    assert!(result.is_err());

    // Restore directory
    std::env::set_current_dir(&original_dir).unwrap();
}

#[test]
fn test_config_project_id() {
    let original_dir = std::env::current_dir().unwrap();
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    let config = BurrowConfig::new();
    let project_id = config.project_id();

    // Should be the directory name or "default"
    assert!(!project_id.is_empty());

    // Restore directory
    std::env::set_current_dir(&original_dir).unwrap();
}
