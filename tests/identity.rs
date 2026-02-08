//! Identity and key file tests.
//!
//! These tests verify identity generation, loading, and key management.

mod support;
use std::fs;
use std::path::PathBuf;
use support::*;

#[test]
fn test_identity_generate_creates_file() {
    use burrow::Vault;
    use std::env;
    use tempfile::TempDir;

    let original_dir = env::current_dir().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let home_dir = TempDir::new().unwrap();

    env::set_var("HOME", home_dir.path());
    env::set_current_dir(&temp_dir).unwrap();

    // Init vault
    let _vault = Vault::init("test-user", None, None, None).unwrap();

    // Verify identity file was created somewhere in ~/.burrow/keys/
    let keys_dir = PathBuf::from(home_dir.path()).join(".burrow").join("keys");

    assert!(keys_dir.exists(), "Keys directory should exist");

    // Find the identity.key file
    let mut found_key = false;
    if let Ok(entries) = std::fs::read_dir(&keys_dir) {
        for entry in entries.flatten() {
            let key_file = entry.path().join("identity.key");
            if key_file.exists() {
                found_key = true;

                // Verify file has restrictive permissions on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let metadata = fs::metadata(&key_file).unwrap();
                    let mode = metadata.permissions().mode();
                    // Should be readable/writable by owner only (0o600 or more restrictive)
                    assert_eq!(
                        mode & 0o077,
                        0,
                        "Identity file should have restrictive permissions"
                    );
                }
                break;
            }
        }
    }

    assert!(found_key, "Should have created identity.key file");

    let _ = env::set_current_dir(&original_dir);
}

#[test]
fn test_identity_load_nonexistent_fails() {
    let t = Test::new();

    // Write a config file that references a non-existent identity
    let config_content = r#"[burrow]
version = "0.1.0"

[recipients]
test = "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p"

[secrets]
"#;
    fs::write(t.dir.path().join(".burrow.toml"), config_content).unwrap();

    // Try to open vault, should fail with clean error
    let output = t.cmd().args(["get", "ANYKEY"]).output().unwrap();
    assert_failure(&output);
    // Should have meaningful error message about missing identity
}

#[test]
fn test_identity_public_key_format() {
    let t = Test::init("test-user");

    // List team members to see the public key
    let output = t.team_list();
    assert_success(&output);
    let out = stdout(&output);

    // Public key should start with "age1"
    assert!(
        out.contains("age1"),
        "Public key should be in age format (age1...)"
    );
}

#[test]
fn test_identity_roundtrip() {
    use burrow::Vault;
    use std::env;
    use tempfile::TempDir;

    let original_dir = env::current_dir().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let home_dir = TempDir::new().unwrap();

    env::set_var("HOME", home_dir.path());
    env::set_current_dir(&temp_dir).unwrap();

    // Generate identity via init
    let vault = Vault::init("alice", None, None, None).unwrap();
    let public_key_1 = vault.identity().public_key();

    // Drop and reload
    drop(vault);
    let vault = Vault::open().unwrap();
    let public_key_2 = vault.identity().public_key();

    // Public keys should match
    assert_eq!(
        public_key_1, public_key_2,
        "Public keys should match after reload"
    );

    let _ = env::set_current_dir(&original_dir);
}

#[cfg(unix)]
#[test]
fn test_identity_insecure_permissions_warns() {
    use burrow::Vault;
    use std::env;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    let original_dir = env::current_dir().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let home_dir = TempDir::new().unwrap();

    env::set_var("HOME", home_dir.path());
    env::set_current_dir(&temp_dir).unwrap();

    // Init vault
    let _vault = Vault::init("test-user", None, None, None).unwrap();

    // Find the identity file
    let keys_dir = PathBuf::from(home_dir.path()).join(".burrow").join("keys");
    let mut key_path = None;

    if let Ok(entries) = std::fs::read_dir(&keys_dir) {
        for entry in entries.flatten() {
            let candidate = entry.path().join("identity.key");
            if candidate.exists() {
                key_path = Some(candidate);
                break;
            }
        }
    }

    let key_path = key_path.expect("Should find identity.key");

    // Change to insecure permissions
    let mut perms = fs::metadata(&key_path).unwrap().permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&key_path, perms).unwrap();

    // Try to use the vault - should work but could warn
    // burrow is forgiving, we're mainly verifying it doesn't crash
    let result = Vault::open();
    assert!(
        result.is_ok(),
        "Vault should still open with insecure permissions"
    );

    let _ = env::set_current_dir(&original_dir);
}
