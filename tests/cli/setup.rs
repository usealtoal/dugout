//! Tests for `dugout setup` and `dugout whoami` commands.

use crate::support::*;
use std::fs;

#[test]
fn test_setup_creates_global_identity() {
    let t = Test::new();

    let output = t.cmd().arg("setup").output().unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "generated identity");

    // Check that ~/.dugout/identity.key exists
    let identity_path = t.home.path().join(".dugout/identity.key");
    assert!(
        identity_path.exists(),
        "~/.dugout/identity.key should exist"
    );

    // Check that ~/.dugout/identity.pub exists
    let pubkey_path = t.home.path().join(".dugout/identity.pub");
    assert!(pubkey_path.exists(), "~/.dugout/identity.pub should exist");

    // Verify public key starts with age1
    let pubkey = fs::read_to_string(&pubkey_path).unwrap();
    assert!(pubkey.trim().starts_with("age1"));
}

#[test]
fn test_setup_idempotent_without_force() {
    let t = Test::new();

    // First setup
    let output = t.cmd().arg("setup").output().unwrap();
    assert_success(&output);

    let pubkey_path = t.home.path().join(".dugout/identity.pub");
    let first_pubkey = fs::read_to_string(&pubkey_path).unwrap();

    // Second setup without --force should warn and not overwrite
    let output = t.cmd().arg("setup").output().unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "already exists");

    let second_pubkey = fs::read_to_string(&pubkey_path).unwrap();
    assert_eq!(
        first_pubkey, second_pubkey,
        "public key should not change without --force"
    );
}

#[test]
fn test_setup_with_force_overwrites() {
    let t = Test::new();

    // First setup
    let output = t.cmd().arg("setup").output().unwrap();
    assert_success(&output);

    let pubkey_path = t.home.path().join(".dugout/identity.pub");
    let first_pubkey = fs::read_to_string(&pubkey_path).unwrap();

    // Second setup with --force should overwrite
    let output = t.cmd().args(["setup", "--force"]).output().unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "generated identity");

    let second_pubkey = fs::read_to_string(&pubkey_path).unwrap();
    assert_ne!(
        first_pubkey, second_pubkey,
        "public key should change with --force"
    );
}

#[test]
fn test_whoami_prints_public_key() {
    let t = Test::new();

    // Setup identity first
    let setup_output = t.cmd().arg("setup").output().unwrap();
    assert_success(&setup_output);

    // Run whoami
    let output = t.cmd().arg("whoami").output().unwrap();
    assert_success(&output);

    let output_str = stdout(&output);
    assert!(output_str.trim().starts_with("age1"));

    // Should match the public key file
    let pubkey_path = t.home.path().join(".dugout/identity.pub");
    let expected_pubkey = fs::read_to_string(&pubkey_path).unwrap();
    assert_eq!(output_str.trim(), expected_pubkey.trim());
}

#[test]
fn test_whoami_without_setup_fails() {
    let t = Test::new();

    let output = t.cmd().arg("whoami").output().unwrap();
    assert_failure(&output);
    assert_stderr_contains(&output, "no identity found");
    assert_stdout_contains(&output, "dugout setup");
}

#[test]
fn test_setup_then_init_uses_global_identity() {
    let t = Test::new();

    // 1. Setup global identity
    let output = t.cmd().arg("setup").output().unwrap();
    assert_success(&output);

    // Get the global public key
    let pubkey_path = t.home.path().join(".dugout/identity.pub");
    let global_pubkey = fs::read_to_string(&pubkey_path).unwrap();

    // 2. Init vault — should use global identity as recipient
    let output = t.cmd().arg("init").output().unwrap();
    assert_success(&output);

    // 3. Set a secret
    let output = t
        .cmd()
        .args(["set", "DB_PASSWORD", "s3cret"])
        .output()
        .unwrap();
    assert_success(&output);

    // 4. Get the secret back — proves we have decrypt access
    let output = t.cmd().args(["get", "DB_PASSWORD"]).output().unwrap();
    assert_success(&output);
    assert_eq!(stdout(&output).trim(), "s3cret");

    // 5. Verify the vault recipient matches global identity
    let output = t.cmd().args(["team", "list", "--json"]).output().unwrap();
    assert_success(&output);
    assert!(
        stdout(&output).contains(global_pubkey.trim()),
        "vault recipient should be the global identity key"
    );
}

#[test]
fn test_setup_then_init_dot_works() {
    let t = Test::new();

    // 1. Setup global identity
    let output = t.cmd().arg("setup").output().unwrap();
    assert_success(&output);

    // 2. Init vault
    let output = t.cmd().arg("init").output().unwrap();
    assert_success(&output);

    // 3. Set a secret
    let output = t.cmd().args(["set", "MY_KEY", "12345"]).output().unwrap();
    assert_success(&output);

    // 4. Create a Makefile so dot can detect the project
    fs::write(t.dir.path().join("Makefile"), "dev:\n\techo test\n").unwrap();

    // 5. dugout . should work without knocking
    let output = t.cmd().arg(".").output().unwrap();
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        !combined.contains("no access"),
        "should have access after setup+init, got: {combined}"
    );
    assert!(
        !combined.contains("knock"),
        "should not suggest knock after setup+init, got: {combined}"
    );
}

#[test]
fn test_setup_output_includes_paths() {
    let t = Test::new();

    let output = t.cmd().arg("setup").output().unwrap();
    assert_success(&output);

    // Should show success and the public key
    assert_stdout_contains(&output, "generated identity");
    let out = stdout(&output);
    assert!(out.contains("age1")); // public key should be displayed
}

#[test]
fn test_open_falls_back_to_global_identity_when_project_key_stale() {
    use age::secrecy::ExposeSecret;

    let t = Test::new();

    // Setup global identity and initialize vault.
    let output = t.cmd().arg("setup").output().unwrap();
    assert_success(&output);
    let output = t.cmd().args(["init", "--name", "alice"]).output().unwrap();
    assert_success(&output);

    // Store a secret while global/project identity are aligned.
    let output = t
        .cmd()
        .args(["set", "FALLBACK_KEY", "fallback_value"])
        .output()
        .unwrap();
    assert_success(&output);

    // Corrupt the project identity with a different key.
    let project_id = t
        .dir
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let project_identity_path = t
        .home
        .path()
        .join(".dugout/keys")
        .join(project_id)
        .join("identity.key");
    let stale_identity = age::x25519::Identity::generate();
    let stale_secret = stale_identity.to_string();
    fs::write(
        &project_identity_path,
        format!("{}\n", stale_secret.expose_secret()),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&project_identity_path, fs::Permissions::from_mode(0o600)).unwrap();
    }

    // Commands should still work via global identity fallback.
    let output = t.cmd().args(["get", "FALLBACK_KEY"]).output().unwrap();
    assert_success(&output);
    assert_eq!(stdout(&output).trim(), "fallback_value");
}
