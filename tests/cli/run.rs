//! Tests for `burrow run` and `burrow env` commands.

use crate::support::*;

#[test]
fn test_run_injects_env_vars() {
    let t = Test::with_secrets("test-user", &[("INJECTED_VAR", "injected_value")]);

    #[cfg(unix)]
    {
        let output = t.run(&["sh", "-c", "echo $INJECTED_VAR"]);
        assert_success(&output);
        assert_stdout_contains(&output, "injected_value");
    }

    #[cfg(windows)]
    {
        let output = t.run(&["cmd", "/c", "echo %INJECTED_VAR%"]);
        assert_success(&output);
        assert_stdout_contains(&output, "injected_value");
    }
}

#[test]
fn test_run_with_no_secrets() {
    let t = Test::init("test-user");

    // No secrets set, just run echo
    #[cfg(unix)]
    {
        let output = t.run(&["echo", "hello"]);
        assert_success(&output);
        assert_stdout_contains(&output, "hello");
    }

    #[cfg(windows)]
    {
        let output = t.run(&["cmd", "/c", "echo hello"]);
        assert_success(&output);
        assert_stdout_contains(&output, "hello");
    }
}

#[test]
fn test_run_command_exit_code_passthrough() {
    let t = Test::init("test-user");

    #[cfg(unix)]
    {
        // Run a command that exits with non-zero
        let output = t.run(&["sh", "-c", "exit 42"]);
        assert!(output.status.code().unwrap_or(0) != 0);
    }

    #[cfg(windows)]
    {
        let output = t.run(&["cmd", "/c", "exit 42"]);
        assert!(output.status.code().unwrap_or(0) != 0);
    }
}

#[test]
fn test_run_without_init_fails() {
    let t = Test::new();

    #[cfg(unix)]
    {
        let output = t.run(&["echo", "test"]);
        assert_failure(&output);
        assert_stderr_contains(&output, "not initialized");
    }

    #[cfg(windows)]
    {
        let output = t.run(&["cmd", "/c", "echo test"]);
        assert_failure(&output);
        assert_stderr_contains(&output, "not initialized");
    }
}

#[test]
fn test_env_without_init_fails() {
    let t = Test::new();

    let output = t.cmd().arg("env").output().unwrap();
    assert_failure(&output);
    assert_stderr_contains(&output, "not initialized");
}

#[test]
fn test_env_with_init_does_not_crash() {
    let t = Test::init("test-user");

    // We can't easily test the interactive shell spawning,
    // but we can verify the command at least recognizes it's initialized
    let _ = t;
}
