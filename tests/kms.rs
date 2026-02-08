//! KMS hybrid mode integration tests.
//!
//! Tests the full CLI flow with hybrid encryption.
//! Uses the mock KMS backend (test builds only).

mod support;
use support::*;

#[test]
fn test_init_with_kms_creates_kms_config() {
    let t = Test::new();

    let output = t
        .cmd()
        .args([
            "init",
            "--no-banner",
            "--name",
            "alice",
            "--kms",
            "arn:aws:kms:us-east-1:123:key/abc",
        ])
        .output()
        .unwrap();
    assert_success(&output);

    // Verify .dugout.toml has [kms] section
    let config = std::fs::read_to_string(t.dir.path().join(".dugout.toml")).unwrap();
    assert!(config.contains("[kms]"), "Config should have [kms] section");
    assert!(
        config.contains("arn:aws:kms"),
        "Config should contain KMS key ARN"
    );
}

#[test]
fn test_hybrid_set_get_roundtrip() {
    let t = Test::new();

    t.cmd()
        .args([
            "init",
            "--no-banner",
            "--name",
            "alice",
            "--kms",
            "arn:aws:kms:us-east-1:123:key/abc",
        ])
        .output()
        .unwrap();

    let output = t.set("SECRET_KEY", "my-secret-value");
    assert_success(&output);

    let output = t.get("SECRET_KEY");
    assert_success(&output);
    assert_stdout_contains(&output, "my-secret-value");
}

#[test]
fn test_hybrid_secrets_are_envelopes() {
    let t = Test::new();

    t.cmd()
        .args([
            "init",
            "--no-banner",
            "--name",
            "alice",
            "--kms",
            "arn:aws:kms:us-east-1:123:key/abc",
        ])
        .output()
        .unwrap();

    t.set("ENVELOPE_TEST", "envelope-value");

    // Read the raw config to verify envelope format
    let config = std::fs::read_to_string(t.dir.path().join(".dugout.toml")).unwrap();
    assert!(
        config.contains("dugout-envelope-v2"),
        "Secrets should be stored as v2 envelopes, got:\n{}",
        config
    );
}

#[test]
fn test_hybrid_list_works() {
    let t = Test::new();

    t.cmd()
        .args([
            "init",
            "--no-banner",
            "--name",
            "alice",
            "--kms",
            "arn:aws:kms:us-east-1:123:key/abc",
        ])
        .output()
        .unwrap();

    t.set("KEY_A", "val_a");
    t.set("KEY_B", "val_b");

    let output = t.list();
    assert_success(&output);
    assert_stdout_contains(&output, "KEY_A");
    assert_stdout_contains(&output, "KEY_B");
}

#[test]
fn test_hybrid_unlock_produces_env_file() {
    let t = Test::new();

    t.cmd()
        .args([
            "init",
            "--no-banner",
            "--name",
            "alice",
            "--kms",
            "arn:aws:kms:us-east-1:123:key/abc",
        ])
        .output()
        .unwrap();

    t.set("DB_URL", "postgres://localhost/prod");

    let output = t.secrets_unlock();
    assert_success(&output);

    let env_content = std::fs::read_to_string(t.dir.path().join(".env")).unwrap();
    assert!(
        env_content.contains("DB_URL=postgres://localhost/prod"),
        "Env file should contain decrypted secret"
    );
}

#[test]
fn test_hybrid_run_injects_secrets() {
    let t = Test::new();

    t.cmd()
        .args([
            "init",
            "--no-banner",
            "--name",
            "alice",
            "--kms",
            "arn:aws:kms:us-east-1:123:key/abc",
        ])
        .output()
        .unwrap();

    t.set("RUN_TEST", "hybrid-run-value");

    let output = t.run(&["printenv", "RUN_TEST"]);
    assert_success(&output);
    assert_stdout_contains(&output, "hybrid-run-value");
}

#[test]
fn test_hybrid_rotate_preserves_secrets() {
    let t = Test::new();

    t.cmd()
        .args([
            "init",
            "--no-banner",
            "--name",
            "alice",
            "--kms",
            "arn:aws:kms:us-east-1:123:key/abc",
        ])
        .output()
        .unwrap();

    t.set("ROTATE_KEY", "rotate-value");

    let output = t.secrets_rotate();
    assert_success(&output);

    // Value should be preserved after rotation
    let output = t.get("ROTATE_KEY");
    assert_success(&output);
    assert_stdout_contains(&output, "rotate-value");
}

#[test]
fn test_hybrid_team_add_reencrypts() {
    let t = Test::new();

    t.cmd()
        .args([
            "init",
            "--no-banner",
            "--name",
            "alice",
            "--kms",
            "arn:aws:kms:us-east-1:123:key/abc",
        ])
        .output()
        .unwrap();

    t.set("TEAM_SECRET", "team-value");

    // Add a team member
    let bob = age::x25519::Identity::generate();
    let bob_pubkey = bob.to_public().to_string();
    let output = t.team_add("bob", &bob_pubkey);
    assert_success(&output);

    // Original user should still be able to read
    let output = t.get("TEAM_SECRET");
    assert_success(&output);
    assert_stdout_contains(&output, "team-value");
}

#[test]
fn test_init_without_kms_stays_age_only() {
    let t = Test::init("alice");

    // Verify NO [kms] section
    let config = std::fs::read_to_string(t.dir.path().join(".dugout.toml")).unwrap();
    assert!(
        !config.contains("[kms]"),
        "Non-KMS init should not have [kms] section"
    );

    // Set and get should work as raw age
    t.set("PLAIN_KEY", "plain-value");
    let output = t.get("PLAIN_KEY");
    assert_success(&output);
    assert_stdout_contains(&output, "plain-value");

    // Verify NOT an envelope
    let config = std::fs::read_to_string(t.dir.path().join(".dugout.toml")).unwrap();
    assert!(
        !config.contains("dugout-envelope-v2"),
        "Non-KMS secrets should not be envelopes"
    );
}
