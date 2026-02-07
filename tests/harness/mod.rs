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
    ///
    /// # Arguments
    ///
    /// * `name` - User name for the recipient
    ///
    /// # Returns
    ///
    /// Command output from `burrow init --no-banner --name <name>`
    pub fn init(&self, name: &str) -> Output {
        self.cmd()
            .args(["init", "--no-banner", "--name", name])
            .output()
            .expect("failed to run burrow init")
    }

    /// Shortcut for `burrow set` command.
    ///
    /// # Arguments
    ///
    /// * `key` - Secret key name
    /// * `val` - Secret value
    ///
    /// # Returns
    ///
    /// Command output from `burrow set <key> <val>`
    pub fn set(&self, key: &str, val: &str) -> Output {
        self.cmd()
            .args(["set", key, val])
            .output()
            .expect("failed to run burrow set")
    }
}

impl Drop for TestEnv {
    /// Restore the original working directory when the test environment is dropped.
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original_dir);
    }
}
