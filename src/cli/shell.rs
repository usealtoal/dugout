//! Interactive shell command.
//!
//! Spawns a subshell with decrypted secrets loaded as environment variables.

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;
use zeroize::Zeroizing;

/// Spawn an interactive shell with secrets loaded.
pub fn execute() -> Result<()> {
    let vault = Vault::open()?;
    let pairs = vault.decrypt_all()?;

    // Determine which shell to use
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    output::success(&format!(
        "Entering shell. {} secrets loaded. Type 'exit' to leave.",
        pairs.len()
    ));
    output::blank();

    let mut cmd = std::process::Command::new(&shell);

    // Inject secrets as environment variables
    for (key, value) in pairs {
        let zeroized_value = Zeroizing::new(value);
        cmd.env(key, zeroized_value.as_str());
    }
    // Secrets are now zeroized as they go out of scope

    let status = cmd.status()?;

    output::blank();
    output::success("Left shell. Secrets cleared.");

    // Return the shell's exit code
    std::process::exit(status.code().unwrap_or(0));
}
