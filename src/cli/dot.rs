//! Dot command.
//!
//! Auto-detect project type and run with secrets injected.

use crate::cli::output;
use crate::core::config::Config;
use crate::core::detect::ProjectKind;
use crate::core::domain::Identity;
use crate::error::Result;

/// Auto-detect project and run with secrets.
pub fn execute() -> Result<()> {
    // Check if vault is initialized
    if !Config::exists() {
        output::blank();
        output::error("no burrow vault found in this directory");
        output::blank();
        output::hint(&format!("run {} first", output::cmd("burrow init")));
        return Err(crate::error::ConfigError::NotInitialized.into());
    }

    // Check if user has access
    let config = Config::load()?;

    // Try to load global identity to check access
    if Identity::has_global()? {
        let pubkey = Identity::load_global_pubkey()?;

        if !config.recipients.values().any(|k| k == &pubkey) {
            output::blank();
            output::error("you don't have access to this vault");
            output::blank();
            output::hint(&format!(
                "run {} to request access",
                output::cmd("burrow knock")
            ));
            return Err(crate::error::ConfigError::NoRecipients.into());
        }
    }

    // Detect project type
    let project_kind = ProjectKind::detect();

    if project_kind.is_none() {
        output::blank();
        output::error("couldn't detect project type");
        output::blank();
        output::hint("no pyproject.toml, package.json, Cargo.toml, go.mod, docker-compose.yml, Makefile, or justfile found");
        output::blank();
        output::hint(&format!(
            "use {} instead",
            output::cmd("burrow run -- <command>")
        ));
        return Err(crate::error::ConfigError::InvalidValue {
            field: "project",
            reason: "could not detect project type".to_string(),
        }
        .into());
    }

    let kind = project_kind.unwrap();
    let command = kind.command();

    output::blank();
    output::kv("detected", kind.display_name());
    output::kv("running", command.join(" "));
    output::blank();

    // Use the existing run infrastructure
    crate::cli::run::execute(&command)
}
