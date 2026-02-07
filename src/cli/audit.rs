//! Audit git history for accidentally committed secrets.

use crate::cli::output;
use crate::core::config::Config;
use crate::error::Result;
use std::process::{Command, Stdio};

/// Scan git history for leaked secrets.
pub fn execute() -> Result<()> {
    // Check if we're in a git repository
    if !is_git_repo() {
        output::warn("Not a git repository");
        output::hint("Run 'git init' to start tracking this project");
        return Ok(());
    }

    output::section("Security Audit");

    let mut warnings = 0;

    // Check 1: .env files in git history
    warnings += check_env_files()?;

    // Check 2: Patterns that look like secrets
    warnings += check_secret_patterns()?;

    // Check 3: .burrow.toml private keys exposed
    warnings += check_burrow_keys()?;

    println!();
    if warnings == 0 {
        output::success("No obvious secrets found in git history");
        output::dimmed("(This is a basic scan. Always review commits manually)");
    } else {
        output::warn(&format!(
            "{} potential issue{} found",
            warnings,
            if warnings == 1 { "" } else { "s" }
        ));
        println!();
        output::hint("Use 'git filter-repo' or 'BFG Repo-Cleaner' to remove sensitive data");
        output::hint("Rotate any exposed credentials immediately");
    }

    Ok(())
}

/// Check if we're in a git repository.
fn is_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check for .env files in git history.
fn check_env_files() -> Result<usize> {
    let output = Command::new("git")
        .args([
            "log",
            "--all",
            "--pretty=format:",
            "--name-only",
            "--diff-filter=A",
        ])
        .output()?;

    let files = String::from_utf8_lossy(&output.stdout);
    let env_files: Vec<_> = files
        .lines()
        .filter(|l| {
            let path = l.trim();
            path == ".env"
                || path.ends_with("/.env")
                || (path.contains(".env.") && !path.ends_with(".env.example"))
        })
        .collect();

    if !env_files.is_empty() {
        output::warn(&format!(
            ".env file{} found in git history:",
            if env_files.len() == 1 { "" } else { "s" }
        ));
        for file in env_files.iter().take(10) {
            output::list_item(file);
        }
        if env_files.len() > 10 {
            output::dimmed(&format!("  ... and {} more", env_files.len() - 10));
        }
        println!();
    }

    Ok(if env_files.is_empty() { 0 } else { 1 })
}

/// Check for secret-like patterns in git history.
fn check_secret_patterns() -> Result<usize> {
    let patterns = [
        "API_KEY=",
        "SECRET=",
        "PASSWORD=",
        "PRIVATE_KEY=",
        "TOKEN=",
        "AWS_SECRET",
        "DB_PASSWORD=",
    ];

    let mut findings = Vec::new();

    for pattern in &patterns {
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
        if !commits.trim().is_empty() {
            findings.push((*pattern, commits.lines().count()));
        }
    }

    if !findings.is_empty() {
        output::warn("Secret-like patterns found in commits:");
        for (pattern, count) in &findings {
            output::list_item(&format!(
                "{} ({} commit{})",
                pattern,
                count,
                if *count == 1 { "" } else { "s" }
            ));
        }
        println!();
        return Ok(1);
    }

    Ok(0)
}

/// Check if .burrow.toml contains unencrypted private keys.
fn check_burrow_keys() -> Result<usize> {
    let config = Config::load().ok();
    if config.is_none() {
        return Ok(0);
    }

    // Check if .burrow.toml is tracked in git
    let output = Command::new("git")
        .args([
            "log",
            "--all",
            "--pretty=format:",
            "--name-only",
            ".burrow.toml",
        ])
        .output()?;

    let tracked = !String::from_utf8_lossy(&output.stdout).trim().is_empty();

    if tracked {
        // This is actually OK - .burrow.toml SHOULD be committed
        // The secrets in it are encrypted
        // Only warn if there are obvious private keys (age-secret-key)
        let toml_content = std::fs::read_to_string(".burrow.toml").unwrap_or_default();
        if toml_content.contains("AGE-SECRET-KEY-") {
            output::warn(".burrow.toml contains unencrypted private key");
            output::list_item("Private keys should NEVER be committed");
            output::list_item("Keep your key in ~/.local/share/burrow/<project>/identity.txt");
            println!();
            return Ok(1);
        }
    }

    Ok(0)
}
