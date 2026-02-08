//! Tests for `dugout knock`, `dugout pending`, and `dugout admit` commands.

use crate::support::*;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_knock_creates_request_file() {
    let t = Test::init("alice");

    // Setup global identity first
    let setup_output = t.cmd().arg("setup").output().unwrap();
    assert_success(&setup_output);

    // Run knock
    let output = t.cmd().args(["knock", "bob"]).output().unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "created access request");

    // Check that request file exists
    let request_path = t.dir.path().join(".dugout/requests/bob.pub");
    assert!(request_path.exists(), "request file should exist");

    // Verify request file contains a valid age public key
    let pubkey = fs::read_to_string(&request_path).unwrap();
    assert!(pubkey.trim().starts_with("age1"));
}

#[test]
fn test_knock_without_global_identity_fails() {
    let t = Test::init("alice");

    let output = t.cmd().args(["knock", "bob"]).output().unwrap();
    assert_failure(&output);
    assert_stderr_contains(&output, "no identity found");
    assert_stdout_contains(&output, "dugout setup");
}

#[test]
fn test_knock_when_already_member() {
    let t = Test::new();

    // Setup global identity first
    let setup_output = t.cmd().arg("setup").output().unwrap();
    assert_success(&setup_output);

    // Get the global public key
    let pubkey_path = t.home.path().join(".dugout/identity.pub");
    let global_pubkey = fs::read_to_string(&pubkey_path).unwrap().trim().to_string();

    // Init vault
    let init_output = t
        .cmd()
        .args(["init", "--no-banner", "--name", "alice"])
        .output()
        .unwrap();
    assert_success(&init_output);

    // Manually add the global public key to recipients to simulate already being a member
    let config_path = t.dir.path().join(".dugout.toml");
    let config_content = fs::read_to_string(&config_path).unwrap();

    // Replace the project-specific key with the global key
    let updated_config = config_content
        .lines()
        .map(|line| {
            if line.contains("age1") && !line.contains(&global_pubkey) {
                // Replace any age key with the global key
                format!("alice = \"{}\"", global_pubkey)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(&config_path, updated_config).unwrap();

    // Try to knock - should warn already a member
    let output = t.cmd().args(["knock", "alice"]).output().unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "already have access");
}

#[test]
fn test_pending_lists_requests() {
    let t = Test::init("alice");

    // Setup global identity and create request
    let setup_output = t.cmd().arg("setup").output().unwrap();
    assert_success(&setup_output);

    let knock_output = t.cmd().args(["knock", "bob"]).output().unwrap();
    assert_success(&knock_output);

    // List pending requests
    let output = t.cmd().arg("pending").output().unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "bob");
    assert_stdout_contains(&output, "age1");
}

#[test]
fn test_pending_when_no_requests() {
    let t = Test::init("alice");

    let output = t.cmd().arg("pending").output().unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "no pending requests");
}

#[test]
fn test_admit_approves_request() {
    let t = Test::init("alice");

    // Setup global identity and create request
    let setup_output = t.cmd().arg("setup").output().unwrap();
    assert_success(&setup_output);

    let knock_output = t.cmd().args(["knock", "bob"]).output().unwrap();
    assert_success(&knock_output);

    // Admit bob
    let output = t.cmd().args(["admit", "bob"]).output().unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "admitted");

    // Request file should be deleted
    let request_path = t.dir.path().join(".dugout/requests/bob.pub");
    assert!(!request_path.exists(), "request file should be deleted");

    // Bob should now be in the team
    let team_output = t.team_list();
    assert_success(&team_output);
    assert_stdout_contains(&team_output, "bob");
}

#[test]
fn test_admit_nonexistent_request_fails() {
    let t = Test::init("alice");

    let output = t.cmd().args(["admit", "nonexistent"]).output().unwrap();
    assert_failure(&output);
    assert_stderr_contains(&output, "no pending request");
}

#[test]
fn test_knock_pending_admit_workflow() {
    let t = Test::init("alice");

    // Setup global identity
    let setup_output = t.cmd().arg("setup").output().unwrap();
    assert_success(&setup_output);

    // Bob knocks
    let knock_output = t.cmd().args(["knock", "bob"]).output().unwrap();
    assert_success(&knock_output);

    // Check pending - should show bob
    let pending_output = t.cmd().arg("pending").output().unwrap();
    assert_success(&pending_output);
    assert_stdout_contains(&pending_output, "bob");

    // Admit bob
    let admit_output = t.cmd().args(["admit", "bob"]).output().unwrap();
    assert_success(&admit_output);

    // Check pending again - should be empty
    let pending_output2 = t.cmd().arg("pending").output().unwrap();
    assert_success(&pending_output2);
    assert_stdout_contains(&pending_output2, "no pending requests");

    // Check team - bob should be there
    let team_output = t.team_list();
    assert_success(&team_output);
    assert_stdout_contains(&team_output, "bob");
}

#[test]
fn test_full_onboarding_with_separate_identities() {
    // Simulates: Alice creates vault, Bob knocks, Alice admits, Bob can decrypt
    let t = Test::new();

    // --- Alice's setup ---
    // Create a separate home for Alice
    let alice_home = tempfile::TempDir::new().unwrap();

    // Alice sets up her global identity
    let output = t
        .cmd()
        .env("HOME", alice_home.path())
        .env("USERPROFILE", alice_home.path())
        .arg("setup")
        .output()
        .unwrap();
    assert_success(&output);

    let alice_pubkey = fs::read_to_string(alice_home.path().join(".dugout/identity.pub"))
        .unwrap()
        .trim()
        .to_string();

    // Alice inits the vault (uses her global identity)
    let output = t
        .cmd()
        .env("HOME", alice_home.path())
        .env("USERPROFILE", alice_home.path())
        .args(["init", "--name", "alice"])
        .output()
        .unwrap();
    assert_success(&output);

    // Alice sets a secret
    let output = t
        .cmd()
        .env("HOME", alice_home.path())
        .env("USERPROFILE", alice_home.path())
        .args(["set", "API_KEY", "super_secret_123"])
        .output()
        .unwrap();
    assert_success(&output);

    // Verify Alice can read it
    let output = t
        .cmd()
        .env("HOME", alice_home.path())
        .env("USERPROFILE", alice_home.path())
        .args(["get", "API_KEY"])
        .output()
        .unwrap();
    assert_success(&output);
    assert_eq!(stdout(&output).trim(), "super_secret_123");

    // --- Bob's setup ---
    let bob_home = tempfile::TempDir::new().unwrap();

    // Bob sets up his global identity
    let output = t
        .cmd()
        .env("HOME", bob_home.path())
        .env("USERPROFILE", bob_home.path())
        .arg("setup")
        .output()
        .unwrap();
    assert_success(&output);

    let bob_pubkey = fs::read_to_string(bob_home.path().join(".dugout/identity.pub"))
        .unwrap()
        .trim()
        .to_string();

    // Keys should be different
    assert_ne!(
        alice_pubkey, bob_pubkey,
        "alice and bob should have different keys"
    );

    // --- Bob knocks ---
    let output = t
        .cmd()
        .env("HOME", bob_home.path())
        .env("USERPROFILE", bob_home.path())
        .args(["knock", "bob"])
        .output()
        .unwrap();
    assert_success(&output);

    // Verify the request file contains Bob's public key
    let request_path = t.dir.path().join(".dugout/requests/bob.pub");
    assert!(request_path.exists());
    let request_key = fs::read_to_string(&request_path).unwrap();
    assert_eq!(request_key.trim(), bob_pubkey);

    // --- Alice admits Bob ---
    let output = t
        .cmd()
        .env("HOME", alice_home.path())
        .env("USERPROFILE", alice_home.path())
        .args(["admit", "bob"])
        .output()
        .unwrap();
    assert_success(&output);

    // Request file should be cleaned up
    assert!(!request_path.exists());

    // --- Verify team has both members ---
    let output = t
        .cmd()
        .env("HOME", alice_home.path())
        .env("USERPROFILE", alice_home.path())
        .args(["team", "list", "--json"])
        .output()
        .unwrap();
    assert_success(&output);
    let team_json = stdout(&output);
    assert!(
        team_json.contains(&alice_pubkey),
        "team should contain alice's key"
    );
    assert!(
        team_json.contains(&bob_pubkey),
        "team should contain bob's key"
    );

    // --- Bob can now decrypt ---
    let output = t
        .cmd()
        .env("HOME", bob_home.path())
        .env("USERPROFILE", bob_home.path())
        .args(["get", "API_KEY"])
        .output()
        .unwrap();
    assert_success(&output);
    assert_eq!(stdout(&output).trim(), "super_secret_123");
}

#[test]
fn test_knock_uses_global_identity_key() {
    // Verify knock writes the GLOBAL identity key, not a project key
    let t = Test::init("alice");

    let setup_output = t.cmd().arg("setup").output().unwrap();
    assert_success(&setup_output);

    let global_pubkey = fs::read_to_string(t.home.path().join(".dugout/identity.pub"))
        .unwrap()
        .trim()
        .to_string();

    let output = t.cmd().args(["knock", "bob"]).output().unwrap();
    assert_success(&output);

    let request_key = fs::read_to_string(t.dir.path().join(".dugout/requests/bob.pub"))
        .unwrap()
        .trim()
        .to_string();

    assert_eq!(
        request_key, global_pubkey,
        "knock should use the global identity key"
    );
}

#[test]
fn test_knock_output_includes_instructions() {
    let t = Test::init("alice");

    let setup_output = t.cmd().arg("setup").output().unwrap();
    assert_success(&setup_output);

    let output = t.cmd().args(["knock", "bob"]).output().unwrap();
    assert_success(&output);

    // Should show success and hint about sharing the request file
    assert_stdout_contains(&output, "created access request");
    assert_stdout_contains(&output, ".dugout/requests/bob.pub");
}
