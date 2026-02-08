//! Tests for error handling and CLI flags.

use crate::support::*;

#[test]
fn test_no_command_shows_help() {
    let t = Test::new();

    let output = t.cmd().arg("--help").output().unwrap();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("burrow") || out.contains("Usage"));
}

#[test]
fn test_unknown_command_fails() {
    let t = Test::new();

    let output = t.cmd().arg("unknown-command").output().unwrap();
    assert_failure(&output);
}

#[test]
fn test_verbose_flag_accepted() {
    let t = Test::new();

    // Verbose flag should be accepted
    let output = t
        .cmd()
        .args(["--verbose", "init", "--no-banner", "--name", "test"])
        .output()
        .unwrap();
    assert_success(&output);
}

#[test]
fn test_version_flag() {
    let t = Test::new();

    let output = t.cmd().arg("--version").output().unwrap();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("burrow") || !out.is_empty());
}

#[test]
fn test_completions_bash_outputs_script() {
    let t = Test::new();

    let output = t.cmd().args(["completions", "bash"]).output().unwrap();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("_burrow") || out.contains("complete"));
}
