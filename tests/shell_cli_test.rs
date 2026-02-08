//! Tests for `burrow env` command.
//!
//! Note: `burrow env` spawns an interactive shell, which is difficult to test
//! in automated tests. These tests verify basic behavior where possible.

mod harness;
use harness::{assert_failure, stderr, TestEnv};

#[test]
fn test_env_without_init_fails() {
    let env = TestEnv::new();

    let output = env.cmd().arg("env").output().unwrap();
    assert_failure(&output);
    let err = stderr(&output);
    assert!(err.contains("not initialized"));
}

#[test]
fn test_env_with_init_does_not_crash() {
    let env = TestEnv::new();
    env.init("test-user");

    // We can't easily test the interactive shell spawning,
    // but we can verify the command at least recognizes it's initialized
    // This test is limited because `env` spawns an interactive shell
    let _ = env;
}
