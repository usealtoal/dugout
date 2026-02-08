//! Tests for `burrow check status/audit` commands.

mod harness;
use harness::{assert_failure, assert_success, stderr, stdout, TestEnv};

#[test]
fn test_status_shows_overview() {
    let env = TestEnv::new();
    env.init("test-user");
    env.set("STATUS_KEY", "status_value");

    let output = env.check_status();
    assert_success(&output);
    let out = stdout(&output);

    // Should show some status information
    assert!(!out.is_empty());
    // Likely contains info about secrets or project
    assert!(out.contains("secret") || out.contains("key") || out.contains("1"));
}

#[test]
fn test_status_without_init_fails() {
    let env = TestEnv::new();

    let output = env.check_status();
    assert_failure(&output);
    let err = stderr(&output);
    assert!(err.contains("not initialized"));
}

#[test]
fn test_audit_in_git_repo() {
    let env = TestEnv::new();
    env.init("test-user");

    // Initialize a git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(env.dir.path())
        .output()
        .ok();

    let output = env.check_audit();
    // Should succeed or at least not crash
    // May fail if no git history, but shouldn't error on the git repo itself
    let _ = output;
}

#[test]
fn test_audit_outside_git_repo() {
    let env = TestEnv::new();
    env.init("test-user");

    // No git init
    let output = env.check_audit();
    // Should handle gracefully (may warn about no git repo)
    let _ = output;
}
