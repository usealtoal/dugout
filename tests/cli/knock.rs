//! Tests for `burrow knock`, `burrow pending`, and `burrow admit` commands.

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
    let request_path = t.dir.path().join(".burrow/requests/bob.pub");
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
    assert_stdout_contains(&output, "burrow setup");
}

#[test]
fn test_knock_when_already_member() {
    let t = Test::new();

    // Setup global identity first
    let setup_output = t.cmd().arg("setup").output().unwrap();
    assert_success(&setup_output);

    // Get the global public key
    let pubkey_path = t.home.path().join(".burrow/identity.pub");
    let global_pubkey = fs::read_to_string(&pubkey_path).unwrap().trim().to_string();

    // Init vault
    let init_output = t
        .cmd()
        .args(["init", "--no-banner", "--name", "alice"])
        .output()
        .unwrap();
    assert_success(&init_output);

    // Manually add the global public key to recipients to simulate already being a member
    let config_path = t.dir.path().join(".burrow.toml");
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
    let request_path = t.dir.path().join(".burrow/requests/bob.pub");
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
fn test_knock_output_includes_instructions() {
    let t = Test::init("alice");

    let setup_output = t.cmd().arg("setup").output().unwrap();
    assert_success(&setup_output);

    let output = t.cmd().args(["knock", "bob"]).output().unwrap();
    assert_success(&output);

    // Should show success and hint about sharing the request file
    assert_stdout_contains(&output, "created access request");
    assert_stdout_contains(&output, ".burrow/requests/bob.pub");
}
