//! Project type detection for auto-running commands.
//!
//! Detects project type from configuration files and provides default commands.

use std::path::Path;

/// Detected project type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    Python,
    Node,
    Rust,
    Go,
    Docker,
    Make,
    Just,
}

impl ProjectKind {
    /// Detect project type from files in the current directory
    ///
    /// Detection priority matches the order: Python → Node → Rust → Go → Docker → Make → Just
    pub fn detect() -> Option<Self> {
        // Check in priority order
        if Path::new("pyproject.toml").exists() {
            return Some(Self::Python);
        }
        if Path::new("package.json").exists() {
            return Some(Self::Node);
        }
        if Path::new("Cargo.toml").exists() {
            return Some(Self::Rust);
        }
        if Path::new("go.mod").exists() {
            return Some(Self::Go);
        }
        if Path::new("docker-compose.yml").exists() || Path::new("docker-compose.yaml").exists() {
            return Some(Self::Docker);
        }
        if Path::new("Makefile").exists() {
            return Some(Self::Make);
        }
        if Path::new("justfile").exists() {
            return Some(Self::Just);
        }

        None
    }

    /// Get the default command for this project type.
    ///
    /// Checks for available tools and common project patterns to pick
    /// the most appropriate command.
    pub fn command(&self) -> Vec<String> {
        match self {
            Self::Python => {
                if which::which("uv").is_ok() {
                    // Check for common entry points
                    if Path::new("manage.py").exists() {
                        vec![
                            "uv".into(),
                            "run".into(),
                            "python".into(),
                            "manage.py".into(),
                            "runserver".into(),
                        ]
                    } else if Path::new("app.py").exists() || Path::new("main.py").exists() {
                        let entry = if Path::new("app.py").exists() {
                            "app.py"
                        } else {
                            "main.py"
                        };
                        vec!["uv".into(), "run".into(), "python".into(), entry.into()]
                    } else {
                        vec!["uv".into(), "run".into(), "python".into()]
                    }
                } else if which::which("python3").is_ok() {
                    vec!["python3".into()]
                } else {
                    vec!["python".into()]
                }
            }
            Self::Node => {
                // Check if a dev script exists in package.json
                let has_dev_script = std::fs::read_to_string("package.json")
                    .map(|s| s.contains("\"dev\""))
                    .unwrap_or(false);

                let tool = if which::which("bun").is_ok() {
                    "bun"
                } else {
                    "npm"
                };
                let script = if has_dev_script { "dev" } else { "start" };
                vec![tool.into(), "run".into(), script.into()]
            }
            Self::Rust => vec!["cargo".into(), "run".into()],
            Self::Go => vec!["go".into(), "run".into(), ".".into()],
            Self::Docker => vec!["docker".into(), "compose".into(), "up".into()],
            Self::Make => vec!["make".into(), "dev".into()],
            Self::Just => vec!["just".into(), "dev".into()],
        }
    }

    /// Display name for user-facing messages
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Python => "python (uv)",
            Self::Node => "node",
            Self::Rust => "rust",
            Self::Go => "go",
            Self::Docker => "docker compose",
            Self::Make => "make",
            Self::Just => "just",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    struct TestContext {
        _tmp: TempDir,
        _original_dir: std::path::PathBuf,
    }

    impl Drop for TestContext {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self._original_dir);
        }
    }

    fn setup_test_dir() -> TestContext {
        let tmp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        TestContext {
            _tmp: tmp,
            _original_dir: original_dir,
        }
    }

    #[test]
    fn test_detect_python() {
        let _ctx = setup_test_dir();
        fs::write("pyproject.toml", "[project]\n").unwrap();
        assert_eq!(ProjectKind::detect(), Some(ProjectKind::Python));
    }

    #[test]
    fn test_detect_node() {
        let _ctx = setup_test_dir();
        fs::write("package.json", "{}").unwrap();
        assert_eq!(ProjectKind::detect(), Some(ProjectKind::Node));
    }

    #[test]
    fn test_detect_rust() {
        let _ctx = setup_test_dir();
        fs::write("Cargo.toml", "[package]\n").unwrap();
        assert_eq!(ProjectKind::detect(), Some(ProjectKind::Rust));
    }

    #[test]
    fn test_detect_priority() {
        let _ctx = setup_test_dir();
        // Create both Python and Node files - should prefer Python
        fs::write("pyproject.toml", "[project]\n").unwrap();
        fs::write("package.json", "{}").unwrap();
        assert_eq!(ProjectKind::detect(), Some(ProjectKind::Python));
    }

    #[test]
    fn test_detect_none() {
        let _ctx = setup_test_dir();
        assert_eq!(ProjectKind::detect(), None);
    }

    #[test]
    fn test_command_generation() {
        let _ctx = setup_test_dir();
        // Python without entry points defaults to uv run python (if uv available)
        let py_cmd = ProjectKind::Python.command();
        assert!(py_cmd[0] == "uv" || py_cmd[0] == "python3" || py_cmd[0] == "python");
        assert_eq!(ProjectKind::Rust.command(), vec!["cargo", "run"]);
    }
}
