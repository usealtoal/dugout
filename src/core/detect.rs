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

    /// Get the default command for this project type
    pub fn command(&self) -> Vec<String> {
        match self {
            Self::Python => vec!["uv".to_string(), "run".to_string()],
            Self::Node => {
                // Prefer bun if available, otherwise npm
                if which::which("bun").is_ok() {
                    vec!["bun".to_string(), "dev".to_string()]
                } else {
                    vec!["npm".to_string(), "run".to_string(), "dev".to_string()]
                }
            }
            Self::Rust => vec!["cargo".to_string(), "run".to_string()],
            Self::Go => vec!["go".to_string(), "run".to_string(), ".".to_string()],
            Self::Docker => vec![
                "docker".to_string(),
                "compose".to_string(),
                "up".to_string(),
            ],
            Self::Make => vec!["make".to_string(), "dev".to_string()],
            Self::Just => vec!["just".to_string(), "dev".to_string()],
        }
    }

    /// Display name for user-facing messages
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
        assert_eq!(ProjectKind::Python.command(), vec!["uv", "run"]);
        assert_eq!(ProjectKind::Rust.command(), vec!["cargo", "run"]);
    }
}
