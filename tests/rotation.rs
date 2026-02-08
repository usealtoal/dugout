//! Key rotation integration tests.
//!
//! Tests the full rotation lifecycle: key generation, archiving,
//! re-encryption, recipient updates, and multi-member scenarios.

mod support;
use support::Test;

use predicates::prelude::*;
use std::fs;

/// Extract a recipient's public key from config TOML content.
fn recipient_key(config: &str, name: &str) -> String {
    for line in config.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(name) && trimmed.contains('=') {
            return trimmed
                .split('=')
                .nth(1)
                .unwrap()
                .trim()
                .trim_matches('"')
                .to_string();
        }
    }
    panic!("recipient '{}' not found in config", name);
}

fn read_config(t: &Test) -> String {
    fs::read_to_string(t.dir.path().join(".dugout.toml")).unwrap()
}

// --- Basic rotation ---

#[test]
fn test_rotate_changes_public_key() {
    let t = Test::with_secrets("alice", &[("SECRET", "value")]);

    let key_before = recipient_key(&read_config(&t), "alice");

    let output = t.secrets_rotate();
    assert!(output.status.success());

    let key_after = recipient_key(&read_config(&t), "alice");

    assert_ne!(
        key_before, key_after,
        "public key should change after rotation"
    );
}

#[test]
fn test_rotate_updates_config_recipient() {
    let t = Test::with_secrets("alice", &[("SECRET", "value")]);

    let key_before = recipient_key(&read_config(&t), "alice");

    t.secrets_rotate();

    let config = read_config(&t);
    assert!(
        !config.contains(&key_before),
        "old public key should be removed from config"
    );

    // New key should be in config and different
    let key_after = recipient_key(&config, "alice");
    assert_ne!(key_before, key_after);
    assert!(
        key_after.starts_with("age1"),
        "new key should be valid age public key"
    );
}

#[test]
fn test_rotate_archives_old_key() {
    let t = Test::with_secrets("alice", &[("SECRET", "value")]);

    t.secrets_rotate();

    // Check that archive directory exists under the home dir
    let home = t.home.path();
    let keys_dir = home.join(".dugout").join("keys");

    // Find the project key directory
    let project_dir = fs::read_dir(&keys_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .expect("should have a project key directory");

    let archive_dir = project_dir.path().join("archive");
    assert!(archive_dir.exists(), "archive directory should exist");

    let archived_keys: Vec<_> = fs::read_dir(&archive_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(archived_keys.len(), 1, "should have one archived key");

    let archived_name = archived_keys[0].file_name().to_string_lossy().to_string();
    assert!(
        archived_name.starts_with("identity.key."),
        "archived key should have timestamp suffix"
    );
}

#[test]
fn test_rotate_secrets_still_decrypt() {
    let secrets = [
        ("DB_URL", "postgres://localhost:5432/mydb"),
        ("API_KEY", "sk-live-abc123def456"),
        ("JWT_SECRET", "super-secret-jwt-key-with-special-chars!@#$%"),
    ];

    let t = Test::with_secrets("alice", &secrets);

    let output = t.secrets_rotate();
    assert!(output.status.success());

    for (key, expected) in &secrets {
        let output = t.get(key);
        assert!(
            output.status.success(),
            "failed to get {} after rotation",
            key
        );
        let actual = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert_eq!(actual, *expected, "secret {} changed after rotation", key);
    }
}

// --- Multi-rotation ---

#[test]
fn test_rotate_twice() {
    let t = Test::with_secrets("alice", &[("SECRET", "unchanged")]);

    let key_before = recipient_key(&read_config(&t), "alice");

    t.secrets_rotate();
    let key_after_first = recipient_key(&read_config(&t), "alice");

    t.secrets_rotate();
    let key_after_second = recipient_key(&read_config(&t), "alice");

    assert_ne!(key_before, key_after_first);
    assert_ne!(key_after_first, key_after_second);

    // Secret still readable
    let output = t.get("SECRET");
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "unchanged");
}

#[test]
fn test_rotate_three_times_creates_archives() {
    let t = Test::with_secrets("alice", &[("S", "v")]);

    // Rotate 3 times — archive count depends on timestamp resolution
    // (chrono format is YYYYMMDD_HHMMSS, so within 1 second they collide)
    // We just verify at least 1 archive exists and secrets survive
    t.secrets_rotate();
    t.secrets_rotate();
    t.secrets_rotate();

    let home = t.home.path();
    let keys_dir = home.join(".dugout").join("keys");
    let project_dir = fs::read_dir(&keys_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .unwrap();

    let archive_dir = project_dir.path().join("archive");
    let count = fs::read_dir(&archive_dir).unwrap().count();
    assert!(count >= 1, "should have at least 1 archived key");

    // Secret still works after 3 rotations
    let output = t.get("S");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "v");
}

// --- Multi-member rotation ---

#[test]
fn test_rotate_preserves_other_recipients() {
    let t = Test::with_secrets("alice", &[("SECRET", "team-value")]);

    let bob = age::x25519::Identity::generate();
    let bob_pub = bob.to_public().to_string();
    let output = t
        .cmd()
        .args(["team", "add", "bob", &bob_pub])
        .output()
        .unwrap();
    assert!(output.status.success());

    let output = t.secrets_rotate();
    assert!(output.status.success());

    // Bob should still be in recipients
    let config = read_config(&t);
    assert!(
        config.contains(&bob_pub),
        "bob's key should survive rotation"
    );

    let output = t.cmd().args(["team", "list"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("alice"));
    assert!(stdout.contains("bob"));
}

#[test]
fn test_rotate_with_three_members() {
    let t = Test::with_secrets("alice", &[("DB", "postgres"), ("KEY", "sk-123")]);

    let bob = age::x25519::Identity::generate();
    let carol = age::x25519::Identity::generate();

    t.cmd()
        .args(["team", "add", "bob", &bob.to_public().to_string()])
        .output()
        .unwrap();
    t.cmd()
        .args(["team", "add", "carol", &carol.to_public().to_string()])
        .output()
        .unwrap();

    let output = t.secrets_rotate();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 secrets re-encrypted"));

    let output = t.get("DB");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "postgres");

    let output = t.get("KEY");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "sk-123");

    let output = t.cmd().args(["team", "list"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("alice"));
    assert!(stdout.contains("bob"));
    assert!(stdout.contains("carol"));
}

// --- Edge cases ---

#[test]
fn test_rotate_empty_vault() {
    let t = Test::init("alice");

    let output = t.secrets_rotate();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0 secrets re-encrypted"));
}

#[test]
fn test_rotate_single_secret() {
    let t = Test::with_secrets("alice", &[("ONLY", "one")]);

    let output = t.secrets_rotate();
    assert!(output.status.success());

    let output = t.get("ONLY");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "one");
}

#[test]
fn test_rotate_unicode_secrets() {
    let t = Test::with_secrets(
        "alice",
        &[
            ("EMOJI", "🔐🗝️🔑"),
            ("CHINESE", "你好世界"),
            ("ARABIC", "مرحبا بالعالم"),
            ("MIXED", "hello 世界 🌍"),
        ],
    );

    let output = t.secrets_rotate();
    assert!(output.status.success());

    let output = t.get("EMOJI");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "🔐🗝️🔑");

    let output = t.get("CHINESE");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "你好世界");

    let output = t.get("ARABIC");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "مرحبا بالعالم"
    );

    let output = t.get("MIXED");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "hello 世界 🌍"
    );
}

#[test]
fn test_rotate_large_secret_values() {
    let large_value = "x".repeat(100_000);
    let t = Test::with_secrets("alice", &[("LARGE", &large_value)]);

    let output = t.secrets_rotate();
    assert!(output.status.success());

    let output = t.get("LARGE");
    assert!(output.status.success());
    let actual = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(actual.len(), 100_000);
}

#[test]
fn test_rotate_no_vault() {
    let t = Test::new();

    t.cmd()
        .args(["secrets", "rotate"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized"));
}

// --- Rotation + operations ---

#[test]
fn test_set_after_rotate() {
    let t = Test::with_secrets("alice", &[("BEFORE", "old")]);

    t.secrets_rotate();

    let output = t.set("AFTER", "new");
    assert!(output.status.success());

    let output = t.get("AFTER");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "new");

    let output = t.get("BEFORE");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "old");
}

#[test]
fn test_import_after_rotate() {
    let t = Test::with_secrets("alice", &[("EXISTING", "keep")]);

    t.secrets_rotate();

    let env_file = t.dir.path().join("new.env");
    fs::write(&env_file, "IMPORTED=fresh\n").unwrap();
    let output = t.secrets_import("new.env");
    assert!(output.status.success());

    let output = t.get("EXISTING");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "keep");

    let output = t.get("IMPORTED");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "fresh");
}

#[test]
fn test_export_after_rotate() {
    let t = Test::with_secrets("alice", &[("KEY_A", "val_a"), ("KEY_B", "val_b")]);

    t.secrets_rotate();

    let output = t.secrets_export();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("KEY_A=val_a"));
    assert!(stdout.contains("KEY_B=val_b"));
}

#[test]
fn test_sync_after_rotate() {
    let t = Test::with_secrets("alice", &[("SECRET", "value")]);

    t.secrets_rotate();

    let output = t.cmd().arg("sync").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        stdout.contains("synced") || stdout.contains("already in sync"),
        "sync should succeed after rotation"
    );
}

#[test]
fn test_team_add_after_rotate() {
    let t = Test::with_secrets("alice", &[("SECRET", "team-secret")]);

    t.secrets_rotate();

    let charlie = age::x25519::Identity::generate();
    let output = t
        .cmd()
        .args(["team", "add", "charlie", &charlie.to_public().to_string()])
        .output()
        .unwrap();
    assert!(output.status.success());

    let output = t.get("SECRET");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "team-secret"
    );
}

#[test]
fn test_rotate_then_remove_member_then_rotate() {
    let t = Test::with_secrets("alice", &[("SECRET", "survive")]);

    let bob = age::x25519::Identity::generate();
    t.cmd()
        .args(["team", "add", "bob", &bob.to_public().to_string()])
        .output()
        .unwrap();

    let output = t.secrets_rotate();
    assert!(output.status.success());

    let output = t.cmd().args(["team", "rm", "bob"]).output().unwrap();
    assert!(output.status.success());

    let output = t.secrets_rotate();
    assert!(output.status.success());

    let output = t.get("SECRET");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "survive");
}
