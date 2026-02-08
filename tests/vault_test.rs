//! Vault API tests.
//!
//! These tests verify the Vault API works correctly through the public interface.
//! Unit tests in src/core/vault.rs already cover crypto roundtrips.

use burrow::Vault;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

struct TestEnv {
    _dir: TempDir,
    _home: TempDir,
    vault: Vault,
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

    let vault = Vault::init("test-user").unwrap();

    TestEnv {
        _dir: temp_dir,
        _home: home_dir,
        vault,
        original_dir,
    }
}

#[test]
fn test_vault_set_and_get() {
    let mut env = setup();

    env.vault.set("TEST_KEY", "test_value", false).unwrap();
    let value = env.vault.get("TEST_KEY").unwrap();
    assert_eq!(value.as_str(), "test_value");
}

#[test]
fn test_vault_set_force() {
    let mut env = setup();

    env.vault.set("KEY", "original", false).unwrap();

    // Without force should fail
    assert!(env.vault.set("KEY", "new", false).is_err());

    // With force should succeed
    env.vault.set("KEY", "new", true).unwrap();
    let value = env.vault.get("KEY").unwrap();
    assert_eq!(value.as_str(), "new");
}

#[test]
fn test_vault_remove() {
    let mut env = setup();

    env.vault.set("TEMP", "value", false).unwrap();
    env.vault.remove("TEMP").unwrap();

    assert!(env.vault.get("TEMP").is_err());
}

#[test]
fn test_vault_list() {
    let mut env = setup();

    env.vault.set("KEY1", "value1", false).unwrap();
    env.vault.set("KEY2", "value2", false).unwrap();
    env.vault.set("KEY3", "value3", false).unwrap();

    let secrets = env.vault.list();
    assert_eq!(secrets.len(), 3);

    let keys: Vec<String> = secrets.iter().map(|s| s.key().to_string()).collect();
    assert!(keys.contains(&"KEY1".to_string()));
    assert!(keys.contains(&"KEY2".to_string()));
    assert!(keys.contains(&"KEY3".to_string()));
}

#[test]
fn test_vault_import() {
    let mut env = setup();

    // Create a test .env file
    fs::write("test.env", "IMPORT1=value1\nIMPORT2=value2\n").unwrap();

    let imported = env.vault.import("test.env").unwrap();
    assert_eq!(imported.len(), 2);

    assert_eq!(env.vault.get("IMPORT1").unwrap().as_str(), "value1");
    assert_eq!(env.vault.get("IMPORT2").unwrap().as_str(), "value2");
}

#[test]
fn test_vault_export() {
    let mut env = setup();

    env.vault.set("EXPORT1", "value1", false).unwrap();
    env.vault.set("EXPORT2", "value2", false).unwrap();

    let exported = env.vault.export().unwrap();
    let exported_str = format!("{}", exported);

    assert!(exported_str.contains("EXPORT1=value1"));
    assert!(exported_str.contains("EXPORT2=value2"));
}

#[test]
fn test_vault_unlock() {
    let mut env = setup();

    env.vault.set("UNLOCK_KEY", "unlock_value", false).unwrap();

    let unlocked = env.vault.unlock().unwrap();
    assert_eq!(unlocked.get("UNLOCK_KEY"), Some("unlock_value"));

    // Should create .env file
    assert!(PathBuf::from(".env").exists());
    let env_content = fs::read_to_string(".env").unwrap();
    assert!(env_content.contains("UNLOCK_KEY=unlock_value"));
}

#[test]
fn test_vault_reencrypt_all() {
    let mut env = setup();

    env.vault.set("REENCRYPT1", "value1", false).unwrap();
    env.vault.set("REENCRYPT2", "value2", false).unwrap();

    env.vault.reencrypt_all().unwrap();

    // Secrets should still be accessible
    assert_eq!(env.vault.get("REENCRYPT1").unwrap().as_str(), "value1");
    assert_eq!(env.vault.get("REENCRYPT2").unwrap().as_str(), "value2");
}

#[test]
fn test_vault_decrypt_all() {
    let mut env = setup();

    env.vault.set("DEC1", "value1", false).unwrap();
    env.vault.set("DEC2", "value2", false).unwrap();

    let decrypted = env.vault.decrypt_all().unwrap();
    assert_eq!(decrypted.len(), 2);

    // Check that all values are present
    let has_dec1 = decrypted.iter().any(|(k, v)| k == "DEC1" && v.as_str() == "value1");
    let has_dec2 = decrypted.iter().any(|(k, v)| k == "DEC2" && v.as_str() == "value2");
    assert!(has_dec1);
    assert!(has_dec2);
}
