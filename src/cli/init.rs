//! Init command.
//!
//! Initializes burrow in the current directory by creating configuration
//! and generating a keypair.

use tracing::info;

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Initialize burrow in the current directory.
pub fn execute(name: Option<String>, no_banner: bool) -> Result<()> {
    if !no_banner {
        crate::cli::banner::print_banner();
    }

    let name = name.unwrap_or_else(whoami::username);

    info!("Initializing for user: {}", name);

    // Generate keypair with spinner
    let sp = output::spinner("Generating keypair...");
    let vault = Vault::init(&name)?;
    output::spinner_success(&sp, "Generated keypair");

    info!("Initialized successfully");

    // Get the first recipient's public key for display
    let recipients = vault.recipients();
    let public_key = recipients
        .first()
        .map(|r| r.public_key())
        .unwrap_or("unknown");

    output::blank();
    output::success("initialized");
    output::kv("recipient", format!("{} ({})", name, &public_key[..20]));
    output::kv(
        "config",
        format!("{} (commit this)", output::path(".burrow.toml")),
    );
    output::kv("key", format!("~/.burrow/keys/{}/", vault.project_id()));
    output::blank();
    output::hint(&format!(
        "Next: {} to add secrets",
        output::cmd("burrow set KEY VALUE")
    ));

    Ok(())
}
