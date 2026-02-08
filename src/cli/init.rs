//! Init command.
//!
//! Initializes burrow in the current directory by creating configuration
//! and generating a keypair.

use tracing::info;

use crate::cli::output;
use crate::core::domain::Identity;
use crate::core::vault::Vault;
use crate::error::Result;

/// Initialize burrow in the current directory.
pub fn execute(
    name: Option<String>,
    no_banner: bool,
    cipher: Option<String>,
    kms_key: Option<String>,
    gcp_key: Option<String>,
) -> Result<()> {
    if !no_banner {
        crate::cli::banner::print_banner();
    }

    // Try to use global identity if it exists and no name was provided
    let name = if let Some(n) = name {
        n
    } else if Identity::has_global()? {
        // Global identity exists - derive name from the config or use username
        output::hint("using global identity from ~/.burrow/identity");
        whoami::username()
    } else {
        whoami::username()
    };

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

    // Generate keypair with spinner
    let sp = output::spinner("generating keypair...");
    let vault = Vault::init(&name, cipher.clone(), kms_key.clone(), gcp_key.clone())?;
    output::spinner_success(&sp, "initialized");

    info!("Initialized successfully");

    // Get the first recipient's public key for display
    let recipients = vault.recipients();
    let public_key = recipients
        .first()
        .map(|r| r.public_key())
        .unwrap_or("unknown");

    output::blank();

    // Show cipher backend if not age
    if let Some(ref c) = cipher {
        if c != "age" {
            output::kv("cipher", c);
        }
    }

    output::kv("recipient", format!("{} ({})", name, &public_key[..20]));
    output::kv(
        "config",
        format!("{} (commit this)", output::path(".burrow.toml")),
    );
    output::kv("key", format!("~/.burrow/keys/{}/", vault.project_id()));
    output::blank();
    output::hint(&format!(
        "run {} to add your first secret",
        output::cmd("burrow set KEY VALUE")
    ));

    Ok(())
}
