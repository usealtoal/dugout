//! Tests for configuration management.

use burrow::core::config::Config;
use tempfile::TempDir;

#[test]
fn test_config_new() {
    let config = Config::new();
    assert_eq!(config.burrow.version, env!("CARGO_PKG_VERSION"));
    assert!(config.recipients.is_empty());
    assert!(config.secrets.is_empty());
}

#[test]
fn test_config_save_and_load() {
    let original_dir = std::env::current_dir().unwrap();
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    // Generate a valid age public key for testing
    use age::x25519;
    let identity = x25519::Identity::generate();
    let public_key = identity.to_public().to_string();

    let mut config = Config::new();
    config
        .recipients
        .insert("alice".to_string(), public_key.clone());
    config
        .secrets
        .insert("KEY".to_string(), "encrypted_value".to_string());

    // Save
    config.save().unwrap();
    assert!(Config::exists());

    // Load
    let loaded = Config::load().unwrap();
    assert_eq!(loaded.recipients.len(), 1);
    assert_eq!(loaded.secrets.len(), 1);
    assert_eq!(loaded.recipients.get("alice").unwrap(), &public_key);
    assert_eq!(loaded.secrets.get("KEY").unwrap(), "encrypted_value");

    // Restore directory
    std::env::set_current_dir(&original_dir).unwrap();
}

#[test]
fn test_config_load_not_initialized() {
    let original_dir = std::env::current_dir().unwrap();
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    let result = Config::load();
    assert!(result.is_err());

    // Restore directory
    std::env::set_current_dir(&original_dir).unwrap();
}

#[test]
fn test_config_project_id() {
    let original_dir = std::env::current_dir().unwrap();
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    let config = Config::new();
    let project_id = config.project_id();

    // Should be the directory name or "default"
    assert!(!project_id.is_empty());

    // Restore directory
    std::env::set_current_dir(&original_dir).unwrap();
}
