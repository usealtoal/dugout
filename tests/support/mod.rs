//! Test support utilities for dugout integration tests.
//!
//! Provides reusable test environment setup and helper commands.

#![allow(dead_code)]

pub mod assertions;
pub mod commands;
pub mod fixtures;
pub mod skip;

#[allow(unused_imports)]
pub use assertions::*;
#[allow(unused_imports)]
pub use fixtures::*;

use tempfile::TempDir;

/// Test environment with isolated temp directories.
///
/// Each test gets its own temporary project dir and home dir.
/// No process-global state is mutated — child processes use `.current_dir()`
/// so tests can safely run in parallel.
pub struct Test {
    /// Temporary directory for the test project
    pub dir: TempDir,
    /// Temporary home directory
    pub home: TempDir,
}

impl Test {
    /// Create a new empty test environment.
    ///
    /// Sets up temporary directories for project and home.
    /// Does NOT change the process working directory — child commands
    /// use `.current_dir()` for isolation instead.
    pub fn new() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let home = TempDir::new().expect("failed to create temp home");

        Self { dir, home }
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
