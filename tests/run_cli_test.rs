//! Tests for `burrow run` command.

mod harness;
use harness::{assert_failure, assert_success, stderr, stdout, TestEnv};

#[test]
fn test_run_injects_env_vars() {
    let env = TestEnv::new();
    env.init("test-user");
    env.set("INJECTED_VAR", "injected_value");

    #[cfg(unix)]
    {
        let output = env.run(&["sh", "-c", "echo $INJECTED_VAR"]);
        assert_success(&output);
        let out = stdout(&output);
        assert!(out.contains("injected_value"));
    }

    #[cfg(windows)]
    {
        let output = env.run(&["cmd", "/c", "echo %INJECTED_VAR%"]);
        assert_success(&output);
        let out = stdout(&output);
        assert!(out.contains("injected_value"));
    }
}

#[test]
fn test_run_with_no_secrets() {
    let env = TestEnv::new();
    env.init("test-user");

    // No secrets set, just run echo
    #[cfg(unix)]
    {
        let output = env.run(&["echo", "hello"]);
        assert_success(&output);
        let out = stdout(&output);
        assert!(out.contains("hello"));
    }

    #[cfg(windows)]
    {
        let output = env.run(&["cmd", "/c", "echo hello"]);
        assert_success(&output);
        let out = stdout(&output);
        assert!(out.contains("hello"));
    }
}

#[test]
fn test_run_command_exit_code_passthrough() {
    let env = TestEnv::new();
    env.init("test-user");

    #[cfg(unix)]
    {
        // Run a command that exits with non-zero
        let output = env.run(&["sh", "-c", "exit 42"]);
        assert!(output.status.code().unwrap_or(0) != 0);
    }

    #[cfg(windows)]
    {
        let output = env.run(&["cmd", "/c", "exit 42"]);
        assert!(output.status.code().unwrap_or(0) != 0);
    }
}

#[test]
fn test_run_without_init_fails() {
    let env = TestEnv::new();

    #[cfg(unix)]
    {
        let output = env.run(&["echo", "test"]);
        assert_failure(&output);
        let err = stderr(&output);
        assert!(err.contains("not initialized"));
    }

    #[cfg(windows)]
    {
        let output = env.run(&["cmd", "/c", "echo test"]);
        assert_failure(&output);
        let err = stderr(&output);
        assert!(err.contains("not initialized"));
    }
}
