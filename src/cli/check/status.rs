//! Status command.
//!
//! Show quick status overview of the burrow project.

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Show quick status overview.
pub fn execute() -> Result<()> {
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
    output::blank();
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
