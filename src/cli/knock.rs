//! Knock command - request access to a vault.

use dialoguer::Input;
use std::io::{self, IsTerminal};

use crate::cli::output;
use crate::core::config::Config;
use crate::core::domain::Identity;
use crate::core::vault::validate_member_name;
use crate::error::Result;

/// Request access to a vault.
pub fn execute(name: Option<String>, vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;

    // Check if global identity exists
    if !Identity::has_global()? {
        output::error("no identity found");
        output::hint("run: dugout setup");
        return Err(
            crate::error::StoreError::NoPrivateKey("~/.dugout/identity".to_string()).into(),
        );
    }

    // Check if already a team member
    if Config::exists_for(vault_name.as_deref()) {
        let config = Config::load_from(vault_name.as_deref())?;
        let pubkey = Identity::load_global_pubkey()?;

        if config.recipients.values().any(|k| k == &pubkey) {
            output::warn("you already have access");
            return Ok(());
        }
    }

    // Prompt for name if not provided
    let name = if let Some(n) = name {
        n
    } else if io::stdin().is_terminal() {
        Input::new()
            .with_prompt("What's your name?")
            .interact_text()?
    } else {
        output::error("name required in non-interactive mode");
        return Err(crate::error::ValidationError::EmptyKey.into());
    };

    validate_member_name(&name)?;

    let pubkey = Identity::load_global_pubkey()?;

    // Create vault-specific requests directory
    let request_dir = crate::core::constants::request_dir(vault_name.as_deref());
    std::fs::create_dir_all(&request_dir)?;

    // Write request file
    let request_path = request_dir.join(format!("{}.pub", name));
    std::fs::write(&request_path, format!("{}\n", pubkey))?;

    output::success("created access request");
    output::hint(&format!("share {} with an admin", request_path.display()));

    Ok(())
}
