//! Command helper methods for Test.

use super::Test;
use assert_cmd::Command;
use std::process::Output;

impl Test {
    /// Create a dugout command with correct environment variables.
    ///
    /// Returns a Command configured with:
    /// - HOME set to the temporary home directory
    /// - Current directory set to the test project directory
    pub fn cmd(&self) -> Command {
        #[allow(deprecated)]
        let mut cmd = Command::cargo_bin("dugout").expect("failed to find dugout binary");
        cmd.env("HOME", self.home.path());
        // Windows uses USERPROFILE instead of HOME for home directory
        cmd.env("USERPROFILE", self.home.path());
        cmd.current_dir(self.dir.path());
        cmd
    }

    /// Shortcut for `dugout init` command.
    pub fn init_cmd(&self, name: &str) -> Output {
        self.cmd()
            .args(["init", "--no-banner", "--name", name])
            .output()
            .expect("failed to run dugout init")
    }

    /// Shortcut for `dugout set` command.
    pub fn set(&self, key: &str, val: &str) -> Output {
        self.cmd()
            .args(["set", key, val])
            .output()
            .expect("failed to run dugout set")
    }

    /// Shortcut for `dugout set --force` command.
    pub fn set_force(&self, key: &str, val: &str) -> Output {
        self.cmd()
            .args(["set", key, val, "--force"])
            .output()
            .expect("failed to run dugout set --force")
    }

    /// Set multiple secrets at once.
    pub fn set_multiple(&self, pairs: &[(&str, &str)]) -> Vec<Output> {
        pairs.iter().map(|(k, v)| self.set(k, v)).collect()
    }

    /// Shortcut for `dugout get` command.
    pub fn get(&self, key: &str) -> Output {
        self.cmd()
            .args(["get", key])
            .output()
            .expect("failed to run dugout get")
    }

    /// Shortcut for `dugout rm` command.
    pub fn rm(&self, key: &str) -> Output {
        self.cmd()
            .args(["rm", key])
            .output()
            .expect("failed to run dugout rm")
    }

    /// Shortcut for `dugout list` command.
    pub fn list(&self) -> Output {
        self.cmd()
            .arg("list")
            .output()
            .expect("failed to run dugout list")
    }

    /// Shortcut for `dugout list --json` command.
    pub fn list_json(&self) -> Output {
        self.cmd()
            .args(["list", "--json"])
            .output()
            .expect("failed to run dugout list --json")
    }

    /// Shortcut for `dugout secrets lock` command.
    pub fn secrets_lock(&self) -> Output {
        self.cmd()
            .args(["secrets", "lock"])
            .output()
            .expect("failed to run dugout secrets lock")
    }

    /// Shortcut for `dugout secrets unlock` command.
    pub fn secrets_unlock(&self) -> Output {
        self.cmd()
            .args(["secrets", "unlock"])
            .output()
            .expect("failed to run dugout secrets unlock")
    }

    /// Shortcut for `dugout secrets import` command.
    pub fn secrets_import(&self, path: &str) -> Output {
        self.cmd()
            .args(["secrets", "import", path])
            .output()
            .expect("failed to run dugout secrets import")
    }

    /// Shortcut for `dugout secrets export` command.
    pub fn secrets_export(&self) -> Output {
        self.cmd()
            .args(["secrets", "export"])
            .output()
            .expect("failed to run dugout secrets export")
    }

    /// Shortcut for `dugout secrets diff` command.
    pub fn secrets_diff(&self) -> Output {
        self.cmd()
            .args(["secrets", "diff"])
            .output()
            .expect("failed to run dugout secrets diff")
    }

    /// Shortcut for `dugout secrets rotate` command.
    pub fn secrets_rotate(&self) -> Output {
        self.cmd()
            .args(["secrets", "rotate"])
            .output()
            .expect("failed to run dugout secrets rotate")
    }

    /// Shortcut for `dugout team add` command.
    pub fn team_add(&self, name: &str, key: &str) -> Output {
        self.cmd()
            .args(["team", "add", name, key])
            .output()
            .expect("failed to run dugout team add")
    }

    /// Shortcut for `dugout team list` command.
    pub fn team_list(&self) -> Output {
        self.cmd()
            .args(["team", "list"])
            .output()
            .expect("failed to run dugout team list")
    }

    /// Shortcut for `dugout team list --json` command.
    pub fn team_list_json(&self) -> Output {
        self.cmd()
            .args(["team", "list", "--json"])
            .output()
            .expect("failed to run dugout team list --json")
    }

    /// Shortcut for `dugout team rm` command.
    pub fn team_rm(&self, name: &str) -> Output {
        self.cmd()
            .args(["team", "rm", name])
            .output()
            .expect("failed to run dugout team rm")
    }

    /// Shortcut for `dugout check status` command.
    pub fn check_status(&self) -> Output {
        self.cmd()
            .args(["check", "status"])
            .output()
            .expect("failed to run dugout check status")
    }

    /// Shortcut for `dugout check audit` command.
    pub fn check_audit(&self) -> Output {
        self.cmd()
            .args(["check", "audit"])
            .output()
            .expect("failed to run dugout check audit")
    }

    /// Shortcut for `dugout run` command.
    pub fn run(&self, command: &[&str]) -> Output {
        let mut cmd = self.cmd();
        cmd.arg("run").arg("--");
        for arg in command {
            cmd.arg(arg);
        }
        cmd.output().expect("failed to run dugout run")
    }
}
