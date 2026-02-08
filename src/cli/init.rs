//! Init command - initialize dugout vault.

use tracing::info;

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Initialize dugout in the current directory.
pub fn execute(name: Option<String>, _no_banner: bool, kms: Option<String>) -> Result<()> {
    let name = name.unwrap_or_else(whoami::username);

    info!("Initializing for user: {}", name);

    let _vault = Vault::init(&name, kms.clone())?;

    if kms.is_some() {
        output::success("initialized vault (hybrid: age + kms)");
    } else {
        output::success("initialized vault");
    }

    info!("Initialized successfully");
    Ok(())
}
