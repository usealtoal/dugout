//! Knock command.
//!
//! Request access to a vault by creating a request file with your public key.

use dialoguer::Input;
use std::io::{self, IsTerminal};

use crate::cli::output;
use crate::core::config::Config;
use crate::core::domain::Identity;
use crate::error::Result;

/// Request access to a vault.
pub fn execute(name: Option<String>) -> Result<()> {
    // Check if global identity exists
    if !Identity::has_global()? {
        output::blank();
        output::error("no global identity found");
        output::blank();
        output::hint(&format!("run {} first", output::cmd("burrow setup")));
        return Err(
            crate::error::StoreError::NoPrivateKey("~/.burrow/identity".to_string()).into(),
        );
    }

    // Check if already a team member
    if Config::exists() {
        let config = Config::load()?;
        let pubkey = Identity::load_global_pubkey()?;

        if config.recipients.values().any(|k| k == &pubkey) {
            output::blank();
            output::warn("you already have access to this vault");
            output::blank();
            let your_name = config
                .recipients
                .iter()
                .find(|(_, k)| *k == &pubkey)
                .map(|(n, _)| n.as_str())
                .unwrap_or("unknown");
            output::kv("your name", your_name);
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
        output::error("name required when not in interactive mode");
        return Err(crate::error::ValidationError::EmptyKey.into());
    };

    let pubkey = Identity::load_global_pubkey()?;

    // Create requests directory
    std::fs::create_dir_all(".burrow/requests")?;

    // Write request file
    let request_path = format!(".burrow/requests/{}.pub", name);
    std::fs::write(&request_path, format!("{}\n", pubkey))?;

    output::blank();
    output::success(&format!(
        "access request created for {}",
        output::key(&name)
    ));
    output::blank();
    output::kv("file", output::path(&request_path));
    output::kv("public key", format!("{}...", &pubkey[..40]));
    output::blank();
    output::hint("commit and push this file, then ask an admin to run:");
    output::note(&format!(
        "  {}",
        output::cmd(&format!("burrow admit {}", name))
    ));

    Ok(())
}
