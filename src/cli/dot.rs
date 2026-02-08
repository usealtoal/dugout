//! Dot command - auto-detect project type and run.

use crate::cli::output;
use crate::core::config::Config;
use crate::core::detect::ProjectKind;
use crate::core::domain::Identity;
use crate::error::Result;

/// Auto-detect project and run with secrets.
pub fn execute() -> Result<()> {
    // Check if vault is initialized
    if !Config::exists() {
        output::error("no vault found");
        output::hint("run: dugout init");
        return Err(crate::error::ConfigError::NotInitialized.into());
    }

    // Check if user has access
    let config = Config::load()?;

    if Identity::has_global()? {
        let pubkey = Identity::load_global_pubkey()?;

        if !config.recipients.values().any(|k| k == &pubkey) {
            output::error("no access to this vault");
            output::hint("run: dugout knock");
            return Err(crate::error::ConfigError::AccessDenied.into());
        }
    }

    // Detect project type
    let project_kind = ProjectKind::detect();

    if project_kind.is_none() {
        output::error("couldn't detect project type");
        output::hint("use: dugout run -- <command>");
        return Err(crate::error::ConfigError::InvalidValue {
            field: "project",
            reason: "could not detect project type".to_string(),
        }
        .into());
    }

    let kind = project_kind.unwrap();
    let command = kind.command();

    // Check if the tool exists before trying to run
    if which::which(&command[0]).is_err() {
        output::error(&format!("{} not found", command[0]));
        output::hint(&format!(
            "install {} or use: dugout run -- <command>",
            command[0]
        ));
        return Err(crate::error::Error::Other(format!(
            "{} not found",
            command[0]
        )));
    }

    crate::cli::run::execute(&command)
}
