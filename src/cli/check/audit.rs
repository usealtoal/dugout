//! Audit command - scan git history for leaked secrets.

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
        return Ok(());
    }

    let findings = audit::scan_git_history()?;

    if findings.is_empty() {
        output::success("no issues found");
    } else {
        output::warn(&format!("{} potential issues found", findings.len()));

        // Show high severity findings
        let high: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == audit::Severity::High)
            .collect();

        for finding in high.iter().take(5) {
            output::list_item(&format!("{}", finding));
        }

        if findings.len() > 5 {
            output::hint(&format!("... and {} more", findings.len() - 5));
        }
    }

    Ok(())
}
