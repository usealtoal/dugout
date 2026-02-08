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

    let vault = Vault::init("test-user", None, None, None).unwrap();

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
    let has_dec1 = decrypted
        .iter()
        .any(|(k, v)| k == "DEC1" && v.as_str() == "value1");
    let has_dec2 = decrypted
        .iter()
        .any(|(k, v)| k == "DEC2" && v.as_str() == "value2");
    assert!(has_dec1);
    assert!(has_dec2);
}

#[test]
fn test_vault_decrypt_all_with_five_secrets() {
    let mut env = setup();

    env.vault.set("SECRET_1", "value1", false).unwrap();
    env.vault.set("SECRET_2", "value2", false).unwrap();
    env.vault.set("SECRET_3", "value3", false).unwrap();
    env.vault.set("SECRET_4", "value4", false).unwrap();
    env.vault.set("SECRET_5", "value5", false).unwrap();

    let decrypted = env.vault.decrypt_all().unwrap();
    assert_eq!(decrypted.len(), 5);

    // Verify all five secrets are present and correct
    for i in 1..=5 {
        let key = format!("SECRET_{}", i);
        let expected_value = format!("value{}", i);
        let found = decrypted
            .iter()
            .any(|(k, v)| k == &key && v.as_str() == expected_value);
        assert!(found, "Missing or incorrect: {}", key);
    }
}

#[test]
fn test_vault_reencrypt_all_preserves_secrets() {
    let mut env = setup();

    env.vault.set("KEY1", "value1", false).unwrap();
    env.vault.set("KEY2", "value2", false).unwrap();
    env.vault.set("KEY3", "value3", false).unwrap();

    env.vault.reencrypt_all().unwrap();

    // All secrets should still be accessible
    assert_eq!(env.vault.get("KEY1").unwrap().as_str(), "value1");
    assert_eq!(env.vault.get("KEY2").unwrap().as_str(), "value2");
    assert_eq!(env.vault.get("KEY3").unwrap().as_str(), "value3");
}

#[test]
fn test_vault_open_existing() {
    let original_dir = env::current_dir().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let home_dir = TempDir::new().unwrap();

    env::set_var("HOME", home_dir.path());
    env::set_current_dir(&temp_dir).unwrap();

    // Init vault
    let mut vault = Vault::init("test-user", None, None, None).unwrap();
    vault
        .set("PERSISTED_KEY", "persisted_value", false)
        .unwrap();

    // Drop the vault
    drop(vault);

    // Open the existing vault
    let opened_vault = Vault::open().unwrap();

    // Verify the secret is still accessible
    assert_eq!(
        opened_vault.get("PERSISTED_KEY").unwrap().as_str(),
        "persisted_value"
    );

    let _ = env::set_current_dir(&original_dir);
}

#[test]
fn test_vault_diff() {
    let mut env = setup();

    env.vault.set("SAME_KEY", "same_value", false).unwrap();
    env.vault.set("CHANGED_KEY", "old_value", false).unwrap();
    env.vault.set("VAULT_ONLY", "vault_value", false).unwrap();

    // Create .env file with different content
    let env_content = "SAME_KEY=same_value\nCHANGED_KEY=new_value\nENV_ONLY=env_value\n";
    fs::write(".env", env_content).unwrap();

    let diff = env.vault.diff(".env").unwrap();
    let entries = diff.entries();

    // Verify we have the expected diff entries
    assert!(entries.iter().any(|e| e.key() == "SAME_KEY"));
    assert!(entries.iter().any(|e| e.key() == "CHANGED_KEY"));
    assert!(entries.iter().any(|e| e.key() == "VAULT_ONLY"));
    assert!(entries.iter().any(|e| e.key() == "ENV_ONLY"));
}

#[test]
fn test_vault_list_returns_secrets() {
    let mut env = setup();

    env.vault.set("LIST_KEY_1", "value1", false).unwrap();
    env.vault.set("LIST_KEY_2", "value2", false).unwrap();
    env.vault.set("LIST_KEY_3", "value3", false).unwrap();

    let secrets = env.vault.list();
    assert_eq!(secrets.len(), 3);

    // Verify we got Secret structs with correct keys
    let keys: Vec<String> = secrets.iter().map(|s| s.key().to_string()).collect();
    assert!(keys.contains(&"LIST_KEY_1".to_string()));
    assert!(keys.contains(&"LIST_KEY_2".to_string()));
    assert!(keys.contains(&"LIST_KEY_3".to_string()));
}

#[test]
fn test_vault_set_returns_secret() {
    let mut env = setup();

    let secret = env.vault.set("RETURN_KEY", "return_value", false).unwrap();

    // Verify set() returns the Secret object with correct key
    assert_eq!(secret.key(), "RETURN_KEY");
}
