//! Interactive shell command - spawn shell with secrets loaded.

use crate::core::vault::Vault;
use crate::error::Result;
use zeroize::Zeroizing;

/// Spawn an interactive shell with secrets loaded.
pub fn execute(vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;
    let v = Vault::open_vault(vault_name.as_deref())?;
    let pairs = v.decrypt_all()?;

    // Determine which shell to use
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    let mut cmd = std::process::Command::new(&shell);

    // Inject secrets as environment variables
    for (key, value) in pairs {
        let zeroized_value = Zeroizing::new(value);
        cmd.env(key, zeroized_value.as_str());
    }

    let status = cmd.status()?;
    std::process::exit(status.code().unwrap_or(0));
}
