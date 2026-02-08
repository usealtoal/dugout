//! Tests for error handling and CLI flags.

mod harness;
use harness::{assert_failure, assert_success, stdout, TestEnv};

#[test]
fn test_no_command_shows_help() {
    let env = TestEnv::new();

    let output = env.cmd().arg("--help").output().unwrap();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("burrow") || out.contains("Usage"));
}

#[test]
fn test_unknown_command_fails() {
    let env = TestEnv::new();

    let output = env.cmd().arg("unknown-command").output().unwrap();
    assert_failure(&output);
}

#[test]
fn test_verbose_flag_accepted() {
    let env = TestEnv::new();

    // Verbose flag should be accepted
    let output = env
        .cmd()
        .args(["--verbose", "init", "--no-banner", "--name", "test"])
        .output()
        .unwrap();
    assert_success(&output);
}

#[test]
fn test_version_flag() {
    let env = TestEnv::new();

    let output = env.cmd().arg("--version").output().unwrap();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("burrow") || !out.is_empty());
}

#[test]
fn test_completions_bash_outputs_script() {
    let env = TestEnv::new();

    let output = env.cmd().args(["completions", "bash"]).output().unwrap();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("_burrow") || out.contains("complete"));
}
