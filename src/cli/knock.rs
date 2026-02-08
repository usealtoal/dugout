//! Knock command - request access to a vault.

use dialoguer::Input;
use std::io::{self, IsTerminal};

use crate::cli::output;
use crate::core::config::Config;
use crate::core::domain::Identity;
use crate::core::vault::validate_member_name;
use crate::error::Result;

/// Request access to a vault.
pub fn execute(name: Option<String>) -> Result<()> {
    // Check if global identity exists
    if !Identity::has_global()? {
        output::error("no identity found");
        output::hint("run: dugout setup");
        return Err(
            crate::error::StoreError::NoPrivateKey("~/.dugout/identity".to_string()).into(),
        );
    }

    // Check if already a team member
    if Config::exists() {
        let config = Config::load()?;
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

    // Create requests directory
    std::fs::create_dir_all(".dugout/requests")?;

    // Write request file
    let request_path = format!(".dugout/requests/{}.pub", name);
    std::fs::write(&request_path, format!("{}\n", pubkey))?;

    output::success("created access request");
    output::hint(&format!("share {} with an admin", request_path));

    Ok(())
}
