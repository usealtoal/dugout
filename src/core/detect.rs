//! Project type detection for auto-running commands.
//!
//! Detects project type from configuration files and provides default commands.

use std::path::Path;

/// Detected project type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    Python,
    Node,
    Deno,
    Rust,
    Go,
    Ruby,
    Elixir,
    Docker,
    Make,
    Just,
}

impl ProjectKind {
    /// Detect project type from files in a directory.
    ///
    /// Detection priority: Python → Deno → Node → Rust → Go → Ruby → Elixir → Docker → Make → Just
    pub fn detect_in(dir: &Path) -> Option<Self> {
        if dir.join("pyproject.toml").exists() {
            return Some(Self::Python);
        }
        if dir.join("deno.json").exists() || dir.join("deno.jsonc").exists() {
            return Some(Self::Deno);
        }
        if dir.join("package.json").exists() {
            return Some(Self::Node);
        }
        if dir.join("Cargo.toml").exists() {
            return Some(Self::Rust);
        }
        if dir.join("go.mod").exists() {
            return Some(Self::Go);
        }
        if dir.join("Gemfile").exists() {
            return Some(Self::Ruby);
        }
        if dir.join("mix.exs").exists() {
            return Some(Self::Elixir);
        }
        if dir.join("docker-compose.yml").exists() || dir.join("docker-compose.yaml").exists() {
            return Some(Self::Docker);
        }
        if dir.join("Makefile").exists() {
            return Some(Self::Make);
        }
        if dir.join("justfile").exists() {
            return Some(Self::Just);
        }

        None
    }

    /// Detect project type from files in the current directory.
    pub fn detect() -> Option<Self> {
        let cwd = std::env::current_dir().ok()?;
        Self::detect_in(&cwd)
    }

    /// Get the default command for this project type.
    ///
    /// Checks for available tools and common project patterns to pick
    /// the most appropriate command.
    pub fn command(&self) -> Vec<String> {
        match self {
            Self::Python => {
                // Check for common entry points first
                let entry = if Path::new("manage.py").exists() {
                    Some(vec!["manage.py".into(), "runserver".into()])
                } else if Path::new("app.py").exists() {
                    Some(vec!["app.py".into()])
                } else if Path::new("main.py").exists() {
                    Some(vec!["main.py".into()])
                } else {
                    None
                };

                // Check for scripts.dev in pyproject.toml (uv/hatch convention)
                let has_dev_script = std::fs::read_to_string("pyproject.toml")
                    .map(|s| {
                        s.contains("[tool.hatch.envs")
                            || (s.contains("scripts") && s.contains("dev"))
                    })
                    .unwrap_or(false);

                if which::which("uv").is_ok() {
                    if has_dev_script {
                        vec!["uv".into(), "run".into(), "dev".into()]
                    } else if let Some(entry_args) = entry {
                        let mut cmd = vec!["uv".into(), "run".into(), "python".into()];
                        cmd.extend(entry_args);
                        cmd
                    } else {
                        // No entry point found — just drop into a Python shell
                        // with secrets loaded (don't try to build the package)
                        vec![
                            "uv".into(),
                            "run".into(),
                            "--no-project".into(),
                            "python".into(),
                        ]
                    }
                } else {
                    let python = if which::which("python3").is_ok() {
                        "python3"
                    } else {
                        "python"
                    };
                    if let Some(entry_args) = entry {
                        let mut cmd = vec![python.into()];
                        cmd.extend(entry_args);
                        cmd
                    } else {
                        vec![python.into()]
                    }
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
            Self::Deno => {
                let has_task = std::fs::read_to_string("deno.json")
                    .or_else(|_| std::fs::read_to_string("deno.jsonc"))
                    .map(|s| s.contains("\"dev\""))
                    .unwrap_or(false);

                if has_task {
                    vec!["deno".into(), "task".into(), "dev".into()]
                } else if Path::new("main.ts").exists() {
                    vec!["deno".into(), "run".into(), "main.ts".into()]
                } else {
                    vec!["deno".into(), "repl".into()]
                }
            }
            Self::Rust => vec!["cargo".into(), "run".into()],
            Self::Go => vec!["go".into(), "run".into(), ".".into()],
            Self::Ruby => {
                if Path::new("config.ru").exists() {
                    // Rails/Rack app
                    let tool = if which::which("bundle").is_ok() {
                        "bundle"
                    } else {
                        "ruby"
                    };
                    vec![tool.into(), "exec".into(), "rails".into(), "server".into()]
                } else {
                    vec!["ruby".into(), "main.rb".into()]
                }
            }
            Self::Elixir => {
                if Path::new("config/runtime.exs").exists() {
                    // Phoenix app
                    vec!["mix".into(), "phx.server".into()]
                } else {
                    vec!["iex".into(), "-S".into(), "mix".into()]
                }
            }
            Self::Docker => vec!["docker".into(), "compose".into(), "up".into()],
            Self::Make => vec!["make".into(), "dev".into()],
            Self::Just => vec!["just".into(), "dev".into()],
        }
    }

    /// Check if there's a .env.example that hints at needed secrets.
    #[allow(dead_code)]
    pub fn env_template_in(dir: &Path) -> Option<Vec<String>> {
        let candidates = [".env.example", ".env.template", ".env.sample"];
        for name in &candidates {
            if let Ok(content) = std::fs::read_to_string(dir.join(name)) {
                let keys: Vec<String> = content
                    .lines()
                    .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
                    .filter_map(|l| l.split('=').next())
                    .map(|k| k.trim().to_string())
                    .filter(|k| !k.is_empty())
                    .collect();
                if !keys.is_empty() {
                    return Some(keys);
                }
            }
        }
        None
    }

    /// Display name for user-facing messages
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Python => "python",
            Self::Node => "node",
            Self::Deno => "deno",
            Self::Rust => "rust",
            Self::Go => "go",
            Self::Ruby => "ruby",
            Self::Elixir => "elixir",
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

    fn tmp() -> TempDir {
        TempDir::new().unwrap()
    }

    #[test]
    fn test_detect_python() {
        let dir = tmp();
        fs::write(dir.path().join("pyproject.toml"), "[project]\n").unwrap();
        assert_eq!(
            ProjectKind::detect_in(dir.path()),
            Some(ProjectKind::Python)
        );
    }

    #[test]
    fn test_detect_node() {
        let dir = tmp();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        assert_eq!(ProjectKind::detect_in(dir.path()), Some(ProjectKind::Node));
    }

    #[test]
    fn test_detect_deno() {
        let dir = tmp();
        fs::write(dir.path().join("deno.json"), "{}").unwrap();
        assert_eq!(ProjectKind::detect_in(dir.path()), Some(ProjectKind::Deno));
    }

    #[test]
    fn test_detect_rust() {
        let dir = tmp();
        fs::write(dir.path().join("Cargo.toml"), "[package]\n").unwrap();
        assert_eq!(ProjectKind::detect_in(dir.path()), Some(ProjectKind::Rust));
    }

    #[test]
    fn test_detect_go() {
        let dir = tmp();
        fs::write(dir.path().join("go.mod"), "module example\n").unwrap();
        assert_eq!(ProjectKind::detect_in(dir.path()), Some(ProjectKind::Go));
    }

    #[test]
    fn test_detect_ruby() {
        let dir = tmp();
        fs::write(
            dir.path().join("Gemfile"),
            "source 'https://rubygems.org'\n",
        )
        .unwrap();
        assert_eq!(ProjectKind::detect_in(dir.path()), Some(ProjectKind::Ruby));
    }

    #[test]
    fn test_detect_elixir() {
        let dir = tmp();
        fs::write(dir.path().join("mix.exs"), "defmodule MyApp do\nend\n").unwrap();
        assert_eq!(
            ProjectKind::detect_in(dir.path()),
            Some(ProjectKind::Elixir)
        );
    }

    #[test]
    fn test_detect_priority() {
        let dir = tmp();
        // Create both Python and Node files — should prefer Python
        fs::write(dir.path().join("pyproject.toml"), "[project]\n").unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        assert_eq!(
            ProjectKind::detect_in(dir.path()),
            Some(ProjectKind::Python)
        );
    }

    #[test]
    fn test_detect_deno_over_node() {
        let dir = tmp();
        // Deno takes priority over Node
        fs::write(dir.path().join("deno.json"), "{}").unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        assert_eq!(ProjectKind::detect_in(dir.path()), Some(ProjectKind::Deno));
    }

    #[test]
    fn test_detect_none() {
        let dir = tmp();
        assert_eq!(ProjectKind::detect_in(dir.path()), None);
    }

    #[test]
    fn test_command_generation() {
        // Python without entry points defaults to uv run python (if uv available)
        let py_cmd = ProjectKind::Python.command();
        assert!(py_cmd[0] == "uv" || py_cmd[0] == "python3" || py_cmd[0] == "python");
        assert_eq!(ProjectKind::Rust.command(), vec!["cargo", "run"]);
        assert_eq!(ProjectKind::Go.command(), vec!["go", "run", "."]);
    }

    #[test]
    fn test_detect_docker_compose_yml() {
        let dir = tmp();
        fs::write(dir.path().join("docker-compose.yml"), "version: '3'\n").unwrap();
        assert_eq!(
            ProjectKind::detect_in(dir.path()),
            Some(ProjectKind::Docker)
        );
    }

    #[test]
    fn test_detect_docker_compose_yaml() {
        let dir = tmp();
        fs::write(dir.path().join("docker-compose.yaml"), "version: '3'\n").unwrap();
        assert_eq!(
            ProjectKind::detect_in(dir.path()),
            Some(ProjectKind::Docker)
        );
    }

    #[test]
    fn test_detect_makefile() {
        let dir = tmp();
        fs::write(dir.path().join("Makefile"), "dev:\n\techo hi\n").unwrap();
        assert_eq!(ProjectKind::detect_in(dir.path()), Some(ProjectKind::Make));
    }

    #[test]
    fn test_detect_justfile() {
        let dir = tmp();
        fs::write(dir.path().join("justfile"), "dev:\n  echo hi\n").unwrap();
        assert_eq!(ProjectKind::detect_in(dir.path()), Some(ProjectKind::Just));
    }

    // Note: env_template tests are in integration tests (tests/cli/dot.rs)
    // because they require set_current_dir which races with parallel unit tests.

    #[test]
    fn test_display_names() {
        assert_eq!(ProjectKind::Python.display_name(), "python");
        assert_eq!(ProjectKind::Node.display_name(), "node");
        assert_eq!(ProjectKind::Deno.display_name(), "deno");
        assert_eq!(ProjectKind::Rust.display_name(), "rust");
        assert_eq!(ProjectKind::Go.display_name(), "go");
        assert_eq!(ProjectKind::Ruby.display_name(), "ruby");
        assert_eq!(ProjectKind::Elixir.display_name(), "elixir");
        assert_eq!(ProjectKind::Docker.display_name(), "docker compose");
        assert_eq!(ProjectKind::Make.display_name(), "make");
        assert_eq!(ProjectKind::Just.display_name(), "just");
    }

    #[test]
    fn test_all_variants_have_commands() {
        // Every project kind should return a non-empty command
        let kinds = [
            ProjectKind::Python,
            ProjectKind::Node,
            ProjectKind::Deno,
            ProjectKind::Rust,
            ProjectKind::Go,
            ProjectKind::Ruby,
            ProjectKind::Elixir,
            ProjectKind::Docker,
            ProjectKind::Make,
            ProjectKind::Just,
        ];
        for kind in &kinds {
            let cmd = kind.command();
            assert!(!cmd.is_empty(), "{:?} returned empty command", kind);
            assert!(!cmd[0].is_empty(), "{:?} returned empty binary name", kind);
        }
    }
}
