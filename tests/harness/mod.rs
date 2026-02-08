//! Test harness utilities for burrow integration tests.
//!
//! Provides reusable test environment setup and helper commands.

use assert_cmd::Command;
use std::env;
use std::path::PathBuf;
use std::process::Output;
use tempfile::TempDir;

/// Test environment with isolated temp directories.
///
/// Automatically sets up temporary directories for the project and home,
/// and restores the original working directory on drop.
pub struct TestEnv {
    /// Temporary directory for the test project
    pub dir: TempDir,
    /// Temporary home directory
    pub home: TempDir,
    /// Original working directory to restore on drop
    original_dir: PathBuf,
}

impl TestEnv {
    /// Create a new test environment.
    ///
    /// Sets up temporary directories and changes the current directory
    /// to the test project directory.
    pub fn new() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let home = TempDir::new().expect("failed to create temp home");
        let original_dir = env::current_dir().expect("failed to get current dir");

        // Change to the test directory
        env::set_current_dir(dir.path()).expect("failed to change to temp dir");

        Self {
            dir,
            home,
            original_dir,
        }
    }

    /// Create a burrow command with correct environment variables.
    ///
    /// Returns a Command configured with:
    /// - HOME set to the temporary home directory
    /// - Current directory set to the test project directory
    pub fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("burrow").expect("failed to find burrow binary");
        cmd.env("HOME", self.home.path());
        cmd.current_dir(self.dir.path());
        cmd
    }

    /// Shortcut for `burrow init` command.
    pub fn init(&self, name: &str) -> Output {
        self.cmd()
            .args(["init", "--no-banner", "--name", name])
            .output()
            .expect("failed to run burrow init")
    }

    /// Shortcut for `burrow set` command.
    pub fn set(&self, key: &str, val: &str) -> Output {
        self.cmd()
            .args(["set", key, val])
            .output()
            .expect("failed to run burrow set")
    }

    /// Shortcut for `burrow set --force` command.
    pub fn set_force(&self, key: &str, val: &str) -> Output {
        self.cmd()
            .args(["set", key, val, "--force"])
            .output()
            .expect("failed to run burrow set --force")
    }

    /// Set multiple secrets at once.
    pub fn set_multiple(&self, pairs: &[(&str, &str)]) -> Vec<Output> {
        pairs.iter().map(|(k, v)| self.set(k, v)).collect()
    }

    /// Shortcut for `burrow get` command.
    pub fn get(&self, key: &str) -> Output {
        self.cmd()
            .args(["get", key])
            .output()
            .expect("failed to run burrow get")
    }

    /// Shortcut for `burrow rm` command.
    pub fn rm(&self, key: &str) -> Output {
        self.cmd()
            .args(["rm", key])
            .output()
            .expect("failed to run burrow rm")
    }

    /// Shortcut for `burrow list` command.
    pub fn list(&self) -> Output {
        self.cmd()
            .arg("list")
            .output()
            .expect("failed to run burrow list")
    }

    /// Shortcut for `burrow list --json` command.
    pub fn list_json(&self) -> Output {
        self.cmd()
            .args(["list", "--json"])
            .output()
            .expect("failed to run burrow list --json")
    }

    /// Shortcut for `burrow secrets lock` command.
    pub fn secrets_lock(&self) -> Output {
        self.cmd()
            .args(["secrets", "lock"])
            .output()
            .expect("failed to run burrow secrets lock")
    }

    /// Shortcut for `burrow secrets unlock` command.
    pub fn secrets_unlock(&self) -> Output {
        self.cmd()
            .args(["secrets", "unlock"])
            .output()
            .expect("failed to run burrow secrets unlock")
    }

    /// Shortcut for `burrow secrets import` command.
    pub fn secrets_import(&self, path: &str) -> Output {
        self.cmd()
            .args(["secrets", "import", path])
            .output()
            .expect("failed to run burrow secrets import")
    }

    /// Shortcut for `burrow secrets export` command.
    pub fn secrets_export(&self) -> Output {
        self.cmd()
            .args(["secrets", "export"])
            .output()
            .expect("failed to run burrow secrets export")
    }

    /// Shortcut for `burrow secrets diff` command.
    pub fn secrets_diff(&self) -> Output {
        self.cmd()
            .args(["secrets", "diff"])
            .output()
            .expect("failed to run burrow secrets diff")
    }

    /// Shortcut for `burrow secrets rotate` command.
    pub fn secrets_rotate(&self) -> Output {
        self.cmd()
            .args(["secrets", "rotate"])
            .output()
            .expect("failed to run burrow secrets rotate")
    }

    /// Shortcut for `burrow team add` command.
    pub fn team_add(&self, name: &str, key: &str) -> Output {
        self.cmd()
            .args(["team", "add", name, key])
            .output()
            .expect("failed to run burrow team add")
    }

    /// Shortcut for `burrow team list` command.
    pub fn team_list(&self) -> Output {
        self.cmd()
            .args(["team", "list"])
            .output()
            .expect("failed to run burrow team list")
    }

    /// Shortcut for `burrow team list --json` command.
    pub fn team_list_json(&self) -> Output {
        self.cmd()
            .args(["team", "list", "--json"])
            .output()
            .expect("failed to run burrow team list --json")
    }

    /// Shortcut for `burrow team rm` command.
    pub fn team_rm(&self, name: &str) -> Output {
        self.cmd()
            .args(["team", "rm", name])
            .output()
            .expect("failed to run burrow team rm")
    }

    /// Shortcut for `burrow check status` command.
    pub fn check_status(&self) -> Output {
        self.cmd()
            .args(["check", "status"])
            .output()
            .expect("failed to run burrow check status")
    }

    /// Shortcut for `burrow check audit` command.
    pub fn check_audit(&self) -> Output {
        self.cmd()
            .args(["check", "audit"])
            .output()
            .expect("failed to run burrow check audit")
    }

    /// Shortcut for `burrow run` command.
    pub fn run(&self, command: &[&str]) -> Output {
        let mut cmd = self.cmd();
        cmd.arg("run").arg("--");
        for arg in command {
            cmd.arg(arg);
        }
        cmd.output().expect("failed to run burrow run")
    }
}

impl Drop for TestEnv {
    /// Restore the original working directory when the test environment is dropped.
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original_dir);
    }
}

/// Assert that a command output was successful.
pub fn assert_success(output: &Output) {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Command failed:\n{}", stderr);
    }
}

/// Assert that a command output failed.
pub fn assert_failure(output: &Output) {
    assert!(
        !output.status.success(),
        "Expected command to fail but it succeeded"
    );
}

/// Get stdout as String.
pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Get stderr as String.
pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}
