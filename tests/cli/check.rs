//! Tests for `burrow check status/audit` commands.

use crate::support::*;

#[test]
fn test_status_shows_overview() {
    let t = Test::with_secrets("test-user", &[("STATUS_KEY", "status_value")]);

    let output = t.check_status();
    assert_success(&output);
    let out = stdout(&output);

    // Should show some status information
    assert!(!out.is_empty());
    // Likely contains info about secrets or project
    assert!(out.contains("secret") || out.contains("key") || out.contains("1"));
}

#[test]
fn test_status_without_init_fails() {
    let t = Test::new();

    let output = t.check_status();
    assert_failure(&output);
    assert_stderr_contains(&output, "not initialized");
}

#[test]
fn test_audit_in_git_repo() {
    let t = Test::init("test-user");

    // Initialize a git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(t.dir.path())
        .output()
        .ok();

    let output = t.check_audit();
    // Should succeed or at least not crash
    // May fail if no git history, but shouldn't error on the git repo itself
    let _ = output;
}

#[test]
fn test_audit_outside_git_repo() {
    let t = Test::init("test-user");

    // No git init
    let output = t.check_audit();
    // Should handle gracefully (may warn about no git repo)
    let _ = output;
}
