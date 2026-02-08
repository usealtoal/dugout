//! Init command - initialize dugout vault.

use tracing::info;

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Initialize dugout in the current directory.
pub fn execute(
    name: Option<String>,
    _no_banner: bool,
    cipher: Option<String>,
    kms_key: Option<String>,
) -> Result<()> {
    let name = name.unwrap_or_else(whoami::username);

    info!("Initializing for user: {}", name);

    let _vault = Vault::init(&name, cipher.clone(), kms_key.clone())?;

    match (cipher.as_deref(), kms_key.is_some()) {
        (Some("gpg"), _) => output::success("initialized vault (gpg)"),
        (_, true) => output::success("initialized vault (hybrid: age + kms)"),
        _ => output::success("initialized vault"),
    }

    info!("Initialized successfully");
    Ok(())
}
