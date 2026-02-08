//! Add command.
//!
//! Interactively add a secret with hidden input.

use std::io::{self, IsTerminal};

use dialoguer::Password;
use tracing::info;

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Add a secret interactively.
pub fn execute(key: &str) -> Result<()> {
    info!("Adding secret: {}", key);

    let mut vault = Vault::open()?;

    // Check if stdin is a pipe
    let value = if !io::stdin().is_terminal() {
        // Read from stdin (piped input)
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    } else {
        // Interactive prompt with hidden input
        Password::new()
            .with_prompt(format!("Value for {}", output::key(key)))
            .interact()?
    };

    if value.is_empty() {
        output::error("value cannot be empty");
        return Err(crate::error::ValidationError::EmptyValue(key.to_string()).into());
    }

    // Check if key already exists
    let force = if vault.list().iter().any(|s| s.key() == key) {
        output::warn(&format!("{} already exists", output::key(key)));
        dialoguer::Confirm::new()
            .with_prompt("Overwrite?")
            .default(false)
            .interact()?
    } else {
        false
    };

    let sp = output::spinner("encrypting...");
    vault.set(key, &value, force)?;
    output::spinner_success(&sp, &format!("added {}", output::key(key)));

    Ok(())
}
