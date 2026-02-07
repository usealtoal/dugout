//! Quick status overview command.

use crate::cli::output;
use crate::core::config::Config;
use crate::error::Result;
use colored::Colorize;

/// Show quick status overview.
pub fn execute() -> Result<()> {
    let config = Config::load()?;

    output::section("Burrow Status");

    // Project name
    let project = config.project_id();
    output::kv("project", &project);

    // Secret count
    let secret_count = config.secrets.len();
    output::kv(
        "secrets",
        format!(
            "{} encrypted",
            if secret_count == 0 {
                "none".dimmed().to_string()
            } else {
                secret_count.to_string().bold().to_string()
            }
        ),
    );

    // Team member count
    let team_count = config.recipients.len();
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
            format!("{} in sync", "✓".green())
        } else if env_count < secret_count {
            format!(
                "{} {} secrets behind",
                "⚠".yellow(),
                secret_count - env_count
            )
        } else {
            format!(
                "{} {} untracked secrets",
                "!".red(),
                env_count - secret_count
            )
        }
    } else {
        format!("{} not found", "✗".red())
    };
    output::kv(".env", env_status);

    // Key file location
    let home = dirs::home_dir().ok_or_else(|| {
        crate::error::Error::Other("unable to determine home directory".to_string())
    })?;
    let key_path = home
        .join(".burrow")
        .join("keys")
        .join(&project)
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
                format!("{} {}", "✓".green(), key_path.display())
            } else {
                format!(
                    "{} {} (insecure permissions: {:o})",
                    "⚠".yellow(),
                    key_path.display(),
                    perms
                )
            }
        }
        #[cfg(not(unix))]
        {
            format!("{} {}", "✓".green(), key_path.display())
        }
    } else {
        format!("{} not found at {}", "✗".red(), key_path.display())
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
            output::cmd("burrow unlock")
        ));
    } else {
        output::dimmed(&format!(
            "Use {} for detailed comparison",
            output::cmd("burrow diff")
        ));
    }

    Ok(())
}
