//! Init command - initialize dugout vault.

use tracing::info;

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Initialize dugout in the current directory.
pub fn execute(
    name: Option<String>,
    _no_banner: bool, // Kept for backwards compatibility but ignored
    cipher: Option<String>,
    kms_key: Option<String>,
    gcp_key: Option<String>,
) -> Result<()> {
    // Use provided name or fall back to username
    let name = name.unwrap_or_else(whoami::username);

    info!("Initializing for user: {}", name);

    // Validate cipher-specific requirements
    if let Some(ref c) = cipher {
        match c.as_str() {
            "aws-kms" if kms_key.is_none() => {
                return Err(crate::error::ConfigError::MissingField {
                    field: "kms_key_id",
                }
                .into());
            }
            "gcp-kms" if gcp_key.is_none() => {
                return Err(crate::error::ConfigError::MissingField {
                    field: "gcp_resource",
                }
                .into());
            }
            _ => {}
        }
    }

    // Generate keypair
    let _vault = Vault::init(&name, cipher.clone(), kms_key.clone(), gcp_key.clone())?;

    output::success("initialized vault");

    info!("Initialized successfully");
    Ok(())
}
