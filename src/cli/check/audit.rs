//! Audit command.
//!
//! Scan git history for leaked secrets.

use crate::cli::output;
use crate::core::domain::audit;
use crate::error::Result;
use std::process::{Command, Stdio};

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

/// Scan git history for leaked secrets.
pub fn execute() -> Result<()> {
    // Check if we're in a git repository
    if !is_git_repo() {
        output::warn("not a git repository");
        output::hint("run 'git init' to start tracking this project");
        return Ok(());
    }

    output::section("Audit");

    let sp = output::spinner("scanning git history...");
    let findings = audit::scan_git_history()?;
    sp.finish_and_clear();

    if findings.is_empty() {
        output::success("no obvious secrets found in git history");
        output::dimmed("(basic scan only, always review commits manually)");
    } else {
        output::warn(&format!(
            "{} potential issue{} found",
            findings.len(),
            if findings.len() == 1 { "" } else { "s" }
        ));
        output::blank();

        // Group findings by severity
        let high: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == audit::Severity::High)
            .collect();
        let medium: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == audit::Severity::Medium)
            .collect();
        let low: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == audit::Severity::Low)
            .collect();

        if !high.is_empty() {
            output::warn(&format!("High severity ({}):", high.len()));
            for finding in high.iter().take(10) {
                output::list_item(&format!("{}", finding));
            }
            if high.len() > 10 {
                output::dimmed(&format!("  ... and {} more", high.len() - 10));
            }
            output::blank();
        }

        if !medium.is_empty() {
            output::dimmed(&format!("Medium severity ({}):", medium.len()));
            for finding in medium.iter().take(5) {
                output::list_item(&format!("{}", finding));
            }
            if medium.len() > 5 {
                output::dimmed(&format!("  ... and {} more", medium.len() - 5));
            }
            output::blank();
        }

        if !low.is_empty() {
            output::dimmed(&format!("Low severity: {} findings (not shown)", low.len()));
            output::blank();
        }

        output::hint("use 'git filter-repo' or 'BFG Repo-Cleaner' to remove sensitive data");
        output::hint("rotate any exposed credentials immediately");
    }

    Ok(())
}
