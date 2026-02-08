//! Sync command integration tests.

mod support;
use support::Test;

use predicates::prelude::*;

// --- CLI tests ---

#[test]
fn test_sync_fresh_vault_no_secrets() {
    let t = Test::init("alice");

    let output = t.cmd().arg("sync").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("already in sync"));
}

#[test]
fn test_sync_after_set_is_already_synced() {
    let t = Test::with_secrets("alice", &[("DB_URL", "postgres://localhost")]);

    let output = t.cmd().arg("sync").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("already in sync"));
}

#[test]
fn test_sync_force_reencrypts() {
    let t = Test::with_secrets(
        "alice",
        &[("DB_URL", "postgres://localhost"), ("API_KEY", "sk-123")],
    );

    let output = t.cmd().args(["sync", "--force"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("synced"));
    assert!(stdout.contains("2 secrets"));
    assert!(stdout.contains("1 recipients"));
}

#[test]
fn test_sync_dry_run_no_changes() {
    let t = Test::with_secrets("alice", &[("SECRET", "value")]);

    let output = t.cmd().args(["sync", "--dry-run"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("already in sync"));
}

#[test]
fn test_sync_dry_run_with_force() {
    let t = Test::with_secrets("alice", &[("SECRET", "value")]);

    let output = t
        .cmd()
        .args(["sync", "--dry-run", "--force"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("would sync"));
}

#[test]
fn test_sync_detects_recipient_change() {
    let t = Test::with_secrets("alice", &[("DB_URL", "postgres://localhost")]);

    // Manually edit the recipients_hash to simulate a recipient change
    let config = std::fs::read_to_string(t.dir.path().join(".dugout.toml")).unwrap();
    let modified = config.replace(
        &extract_hash(&config),
        "recipients_hash = \"0000000000000000000000000000000000000000000000000000000000000000\"",
    );
    std::fs::write(t.dir.path().join(".dugout.toml"), modified).unwrap();

    let output = t.cmd().arg("sync").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("synced"));
    assert!(stdout.contains("1 secrets"));
}

#[test]
fn test_sync_detects_missing_hash() {
    let t = Test::with_secrets("alice", &[("SECRET", "value")]);

    // Remove the recipients_hash line to simulate old config format
    let config = std::fs::read_to_string(t.dir.path().join(".dugout.toml")).unwrap();
    let modified = remove_hash_line(&config);
    std::fs::write(t.dir.path().join(".dugout.toml"), modified).unwrap();

    let output = t.cmd().arg("sync").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("synced"));
}

#[test]
fn test_sync_preserves_secret_values() {
    let t = Test::with_secrets(
        "alice",
        &[
            ("DB_URL", "postgres://localhost"),
            ("API_KEY", "sk-secret-123"),
        ],
    );

    // Force sync
    let output = t.cmd().args(["sync", "--force"]).output().unwrap();
    assert!(output.status.success());

    // Verify secrets are still readable
    let output = t.get("DB_URL");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "postgres://localhost"
    );

    let output = t.get("API_KEY");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "sk-secret-123"
    );
}

#[test]
fn test_sync_idempotent() {
    let t = Test::with_secrets("alice", &[("SECRET", "value")]);

    // Sync twice
    let output1 = t.cmd().args(["sync", "--force"]).output().unwrap();
    assert!(output1.status.success());

    let output2 = t.cmd().arg("sync").output().unwrap();
    let stdout = String::from_utf8_lossy(&output2.stdout);
    assert!(output2.status.success());
    assert!(stdout.contains("already in sync"));
}

#[test]
fn test_sync_after_team_add() {
    let t = Test::with_secrets("alice", &[("DB_URL", "postgres://localhost")]);

    // Generate a second identity
    let bob_identity = age::x25519::Identity::generate();
    let bob_pubkey = bob_identity.to_public().to_string();

    // Add bob as recipient
    let output = t
        .cmd()
        .args(["team", "add", "bob", &bob_pubkey])
        .output()
        .unwrap();
    assert!(output.status.success());

    // team add already re-encrypts, so sync should say in sync
    let output = t.cmd().arg("sync").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("already in sync"));
}

#[test]
fn test_sync_no_vault() {
    let t = Test::new();

    t.cmd()
        .arg("sync")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized"));
}

#[test]
fn test_sync_many_secrets() {
    let t = Test::init("alice");

    // Add 20 secrets
    for i in 0..20 {
        let output = t.set(&format!("SECRET_{}", i), &format!("value_{}", i));
        assert!(output.status.success());
    }

    // Force sync all
    let output = t.cmd().args(["sync", "--force"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("20 secrets"));

    // Verify a few
    for i in [0, 10, 19] {
        let output = t.get(&format!("SECRET_{}", i));
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            format!("value_{}", i)
        );
    }
}

// --- Helpers ---

/// Extract the recipients_hash line from config content.
fn extract_hash(config: &str) -> String {
    config
        .lines()
        .find(|l| l.contains("recipients_hash"))
        .unwrap_or("")
        .to_string()
}

/// Remove the recipients_hash line from config content.
fn remove_hash_line(config: &str) -> String {
    config
        .lines()
        .filter(|l| !l.contains("recipients_hash"))
        .collect::<Vec<_>>()
        .join("\n")
}
