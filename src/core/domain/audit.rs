//! Audit types.
//!
//! Domain types for git history security scanning results.

use crate::error::Result;
use std::process::Command;

/// Severity level for audit findings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Low confidence match (common variable name).
    Low,
    /// Medium confidence (looks like a key pattern).
    Medium,
    /// High confidence (matches known secret formats).
    High,
}

/// A single finding from a git history audit.
#[derive(Debug, Clone)]
pub struct Finding {
    /// Git commit hash.
    pub commit: String,
    /// File path where the finding was detected.
    pub file: String,
    /// Line number (if available).
    pub line: Option<usize>,
    /// The pattern that matched.
    pub pattern: String,
    /// How confident we are this is a real leak.
    pub severity: Severity,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let commit_short = if self.commit.len() >= 8 {
            &self.commit[..8]
        } else {
            &self.commit
        };
        write!(
            f,
            "[{:?}] {} in {} (commit {})",
            self.severity, self.pattern, self.file, commit_short
        )
    }
}

/// Scan git history for potential secret leaks.
///
/// # Returns
///
/// Vector of findings sorted by severity (highest first).
///
/// # Errors
///
/// Returns error if git commands fail or if not in a git repository.
pub fn scan_git_history() -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    // Check for .env files
    findings.extend(scan_env_files()?);

    // Check for secret patterns
    findings.extend(scan_secret_patterns()?);

    // Sort by severity (highest first)
    findings.sort_by(|a, b| b.severity.cmp(&a.severity));

    Ok(findings)
}

/// Scan for .env files in git history.
fn scan_env_files() -> Result<Vec<Finding>> {
    let output = Command::new("git")
        .args([
            "log",
            "--all",
            "--pretty=format:%H",
            "--name-only",
            "--diff-filter=A",
        ])
        .output()?;

    let content = String::from_utf8_lossy(&output.stdout);
    let mut findings = Vec::new();
    let mut current_commit = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check if this is a commit hash (40 hex chars)
        if trimmed.len() == 40 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            current_commit = trimmed.to_string();
        } else if !current_commit.is_empty() {
            // Check if this is an .env file
            if trimmed == ".env"
                || trimmed.ends_with("/.env")
                || (trimmed.contains(".env.") && !trimmed.ends_with(".env.example"))
            {
                findings.push(Finding {
                    commit: current_commit.clone(),
                    file: trimmed.to_string(),
                    line: None,
                    pattern: ".env file".to_string(),
                    severity: Severity::High,
                });
            }
        }
    }

    Ok(findings)
}

/// Scan for secret-like patterns in git history.
fn scan_secret_patterns() -> Result<Vec<Finding>> {
    let patterns = [
        ("API_KEY=", Severity::Medium),
        ("SECRET=", Severity::Medium),
        ("PASSWORD=", Severity::Medium),
        ("PRIVATE_KEY=", Severity::High),
        ("TOKEN=", Severity::Medium),
        ("AWS_SECRET", Severity::High),
        ("DB_PASSWORD=", Severity::Medium),
    ];

    let mut findings = Vec::new();

    for (pattern, severity) in &patterns {
        let output = Command::new("git")
            .args([
                "log",
                "-S",
                pattern,
                "--all",
                "--pretty=format:%H",
                "--",
                "*.env*",
                "*.toml",
                "*.yaml",
                "*.yml",
                "*.json",
            ])
            .output()?;

        let commits = String::from_utf8_lossy(&output.stdout);
        for commit in commits.lines() {
            let commit = commit.trim();
            if !commit.is_empty() {
                findings.push(Finding {
                    commit: commit.to_string(),
                    file: "config file".to_string(),
                    line: None,
                    pattern: pattern.to_string(),
                    severity: severity.clone(),
                });
            }
        }
    }

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High > Severity::Low);
    }

    #[test]
    fn test_finding_display() {
        let finding = Finding {
            commit: "abc123def456".to_string(),
            file: "config/secrets.yml".to_string(),
            line: Some(42),
            pattern: "AWS_SECRET_KEY".to_string(),
            severity: Severity::High,
        };

        let display = format!("{}", finding);
        assert!(display.contains("High"));
        assert!(display.contains("AWS_SECRET_KEY"));
        assert!(display.contains("config/secrets.yml"));
        assert!(display.contains("abc123de")); // First 8 chars of commit
    }

    #[test]
    fn test_finding_without_line() {
        let finding = Finding {
            commit: "abc123".to_string(),
            file: "test.txt".to_string(),
            line: None,
            pattern: "password".to_string(),
            severity: Severity::Low,
        };

        // Should not panic when formatting
        let _display = format!("{}", finding);
    }
}
