//! Dot command - auto-detect project type and run.

use crate::cli::output;
use crate::core::config::Config;
use crate::core::detect::ProjectKind;
use crate::core::domain::Identity;
use crate::error::Result;

/// Auto-detect project and run with secrets.
pub fn execute(vault: Option<String>) -> Result<()> {
    // dot always uses default vault unless explicit
    let vault_name = crate::cli::resolve::resolve_vault_default(vault.as_deref());

    // Check if vault is initialized
    if !Config::exists_for(vault_name.as_deref()) {
        output::error("no vault found");
        output::hint("run: dugout init");
        return Err(crate::error::ConfigError::NotInitialized.into());
    }

    // Check if user has access
    let config = Config::load_from(vault_name.as_deref())?;

    if Identity::has_global()? {
        let pubkey = Identity::load_global_pubkey()?;

        if !config.recipients.values().any(|k| k == &pubkey) {
            output::error("no access to this vault");
            output::hint("run: dugout knock");
            return Err(crate::error::ConfigError::AccessDenied.into());
        }
    }

    // Detect project type
    let kind = match ProjectKind::detect() {
        Some(k) => k,
        None => {
            output::error("couldn't detect project type");
            output::hint("use: dugout run -- <command>");
            return Err(crate::error::ConfigError::InvalidValue {
                field: "project",
                reason: "could not detect project type".to_string(),
            }
            .into());
        }
    };
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

    // Show what we're doing (one concise line)
    let secrets_count = config.secrets.len();
    let cmd_display = command.join(" ");
    output::success(&format!(
        "{} project, {} secrets → {}",
        kind.display_name(),
        secrets_count,
        cmd_display
    ));

    crate::cli::run::execute_with_vault(&command, vault_name)
}
