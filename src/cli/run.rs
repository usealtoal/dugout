//! Run command.
//!
//! Executes a command with decrypted secrets injected as environment variables.

use crate::core::vault::Vault;
use crate::error::Result;
use zeroize::Zeroizing;

/// Run a command with secrets injected as environment variables.
pub fn execute(command: &[String]) -> Result<()> {
    let vault = Vault::open()?;
    let exit_code = run_with_secrets(&vault, command)?;
    std::process::exit(exit_code);
}

/// Run a command with decrypted secrets as environment variables.
fn run_with_secrets(vault: &Vault, command: &[String]) -> Result<i32> {
    if command.is_empty() {
        return Err(crate::error::Error::Other(
            "no command specified".to_string(),
        ));
    }

    let pairs = vault.decrypt_all()?;

    let mut cmd = std::process::Command::new(&command[0]);
    cmd.args(&command[1..]);

    // Inject secrets as environment variables
    // Use Zeroizing to ensure secrets are wiped from memory after use
    for (key, value) in pairs {
        let zeroized_value = Zeroizing::new(value);
        cmd.env(key, zeroized_value.as_str());
    }
    // Secrets are now zeroized as they go out of scope

    let status = cmd.status()?;
    // If the process was terminated by a signal, return 128 + signal number convention
    // Otherwise return the actual exit code, or 1 if unavailable
    Ok(status.code().unwrap_or(1))
}
