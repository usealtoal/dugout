//! Initialize burrow in the current directory.

use colored::Colorize;

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

    println!("{}", "burrow initialized".green().bold());
    println!("  recipient: {} ({})", name, &public_key[..20]);
    println!("  config:    .burrow.toml (commit this)");
    println!("  key:       ~/.burrow/keys/{}/", project_id);
    println!();
    println!("Next: {} to add secrets", "burrow set KEY VALUE".cyan());

    Ok(())
}
