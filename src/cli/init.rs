//! Init command - initialize dugout vault.

use tracing::info;

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Initialize dugout in the current directory.
pub fn execute(
    name: Option<String>,
    _no_banner: bool,
    kms: Option<String>,
    vault: Option<String>,
) -> Result<()> {
    // Use resolve_vault_default since init creates a specific vault
    let vault_name = crate::cli::resolve::resolve_vault_default(vault.as_deref())?;

    let name = name.unwrap_or_else(whoami::username);

    info!("Initializing for user: {}", name);

    let vault_display = vault_name
        .as_ref()
        .map(|n| format!(".dugout.{}.toml", n))
        .unwrap_or_else(|| ".dugout.toml".to_string());

    let _vault = Vault::init_vault(vault_name.as_deref(), &name, kms.clone())?;

    if kms.is_some() {
        output::success(&format!("initialized {} (hybrid: age + kms)", vault_display));
    } else {
        output::success(&format!("initialized {}", vault_display));
    }

    info!("Initialized successfully");
    Ok(())
}
