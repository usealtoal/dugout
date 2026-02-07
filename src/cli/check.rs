//! Diagnostic and check commands.
//!
//! Status overview and git history auditing.

use crate::cli::output;
use crate::core::audit;
use crate::core::vault::Vault;
use crate::error::Result;
use std::process::{Command, Stdio};

/// Show quick status overview.
pub fn status() -> Result<()> {
    let vault = Vault::open()?;

    output::section("Burrow Status");

    // Project name
    let project = vault.project_id();
    output::kv("project", project);

    // Secret count
    let secret_count = vault.list().len();
    let secrets_display = if secret_count == 0 {
        String::from("none encrypted")
    } else {
        format!("{} encrypted", secret_count)
    };
    output::kv("secrets", secrets_display);

    // Team member count
    let team_count = vault.recipients().len();
    output::kv(
        "team",
        format!(
            "{} member{}",
            team_count,
            if team_count == 1 { "" } else { "s" }
        ),
    );

    // Check .env sync status
    let env_path = std::path::Path::new(".env");
    let env_status = if env_path.exists() {
        // Check if in sync
        let env_content = std::fs::read_to_string(env_path)?;
        let env_count = env_content
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .count();

        if env_count == secret_count {
            String::from("✓ in sync")
        } else if env_count < secret_count {
            format!("⚠ {} secrets behind", secret_count - env_count)
        } else {
            format!("⚠ {} untracked secrets", env_count - secret_count)
        }
    } else {
        String::from("✗ not found")
    };
    output::kv(".env", env_status);

    // Key file location
    let home = dirs::home_dir().ok_or_else(|| {
        crate::error::Error::Other("unable to determine home directory".to_string())
    })?;
    let key_path = home
        .join(".burrow")
        .join("keys")
        .join(project)
        .join("identity.key");

    let key_status = if key_path.exists() {
        // Check permissions (should be 0600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&key_path)?;
            let mode = metadata.permissions().mode();
            let perms = mode & 0o777;

            if perms == 0o600 {
                format!("✓ {}", key_path.display())
            } else {
                format!(
                    "⚠ {} (insecure permissions: {:o})",
                    key_path.display(),
                    perms
                )
            }
        }
        #[cfg(not(unix))]
        {
            format!("✓ {}", key_path.display())
        }
    } else {
        format!("✗ not found at {}", key_path.display())
    };
    output::kv("key file", key_status);

    // Suggestions
    println!();
    if secret_count == 0 {
        output::hint(&format!(
            "Add your first secret with {}",
            output::cmd("burrow set KEY value")
        ));
    } else if !env_path.exists() {
        output::hint(&format!(
            "Create .env file with {}",
            output::cmd("burrow secrets unlock")
        ));
    } else {
        output::dimmed(&format!(
            "Use {} for detailed comparison",
            output::cmd("burrow secrets diff")
        ));
    }

    Ok(())
}

/// Scan git history for leaked secrets.
pub fn audit() -> Result<()> {
    // Check if we're in a git repository
    if !is_git_repo() {
        output::warn("Not a git repository");
        output::hint("Run 'git init' to start tracking this project");
        return Ok(());
    }

    output::section("Security Audit");

    let findings = audit::scan_git_history()?;

    if findings.is_empty() {
        output::success("No obvious secrets found in git history");
        output::dimmed("(This is a basic scan. Always review commits manually)");
    } else {
        output::warn(&format!(
            "{} potential issue{} found",
            findings.len(),
            if findings.len() == 1 { "" } else { "s" }
        ));
        println!();

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
            println!();
        }

        if !medium.is_empty() {
            output::dimmed(&format!("Medium severity ({}):", medium.len()));
            for finding in medium.iter().take(5) {
                output::list_item(&format!("{}", finding));
            }
            if medium.len() > 5 {
                output::dimmed(&format!("  ... and {} more", medium.len() - 5));
            }
            println!();
        }

        if !low.is_empty() {
            output::dimmed(&format!("Low severity: {} findings (not shown)", low.len()));
            println!();
        }

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
