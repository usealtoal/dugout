use std::process::Command;

use crate::config::BurrowConfig;
use crate::error::Result;
use crate::secrets;

/// Run a command with secrets injected as environment variables
/// Secrets are decrypted in-memory and never written to disk
pub fn run_with_secrets(config: &BurrowConfig, command: &[String]) -> Result<i32> {
    if command.is_empty() {
        return Err(crate::error::BurrowError::Config(
            "no command specified".to_string(),
        ));
    }

    let pairs = secrets::decrypt_all(config)?;

    let status = Command::new(&command[0])
        .args(&command[1..])
        .envs(pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())))
        .status()?;

    Ok(status.code().unwrap_or(1))
}
