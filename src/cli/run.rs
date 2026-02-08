//! Run command - execute a command with secrets injected.

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
    for (key, value) in pairs {
        let zeroized_value = Zeroizing::new(value);
        cmd.env(key, zeroized_value.as_str());
    }

    let status = cmd.status()?;
    Ok(status.code().unwrap_or(1))
}
