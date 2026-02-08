//! Test support utilities for dugout integration tests.
//!
//! Provides reusable test environment setup and helper commands.

pub mod assertions;
pub mod commands;
pub mod fixtures;

pub use assertions::*;
pub use fixtures::*;

use std::env;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test environment with isolated temp directories.
///
/// Automatically sets up temporary directories for the project and home,
/// and restores the original working directory on drop.
pub struct Test {
    /// Temporary directory for the test project
    pub dir: TempDir,
    /// Temporary home directory
    pub home: TempDir,
    /// Original working directory to restore on drop
    original_dir: PathBuf,
}

impl Test {
    /// Create a new empty test environment.
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

    /// Create a test environment with vault initialized.
    pub fn init(name: &str) -> Self {
        let t = Self::new();
        let output = t.init_cmd(name);
        assert!(
            output.status.success(),
            "Failed to initialize vault: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        t
    }

    /// Create a test environment with vault initialized and secrets set.
    pub fn with_secrets(name: &str, secrets: &[(&str, &str)]) -> Self {
        let t = Self::init(name);
        for (k, v) in secrets {
            let output = t.set(k, v);
            assert!(
                output.status.success(),
                "Failed to set secret {}: {}",
                k,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        t
    }
}

impl Drop for Test {
    /// Restore the original working directory when the test environment is dropped.
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original_dir);
    }
}
