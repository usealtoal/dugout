//! Initialize burrow in the current directory.

use crate::cli::output;
use crate::core::config::{self, Config};
use crate::core::store;
use crate::error::Result;

/// Initialize burrow in the current directory.
pub fn execute(name: Option<String>, no_banner: bool) -> Result<()> {
    if Config::exists() {
        return Err(crate::error::ConfigError::AlreadyInitialized.into());
    }

    if !no_banner {
        crate::cli::banner::print_banner();
    }

    let name = name.unwrap_or_else(whoami::username);

    let mut config = Config::new();
    let project_id = config.project_id();

    let public_key = store::generate_keypair(&project_id)?;
    config.recipients.insert(name.clone(), public_key.clone());
    config.save()?;

    config::ensure_gitignore()?;

    println!();
    output::success("burrow initialized");
    output::kv("recipient", format!("{} ({})", name, &public_key[..20]));
    output::kv(
        "config",
        format!("{} (commit this)", output::path(".burrow.toml")),
    );
    output::kv("key", format!("~/.burrow/keys/{}/", project_id));
    println!();
    output::hint(&format!(
        "Next: {} to add secrets",
        output::cmd("burrow set KEY VALUE")
    ));

    Ok(())
}
