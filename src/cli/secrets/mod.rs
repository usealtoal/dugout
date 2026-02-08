//! Secret management commands.
//!
//! Implements set, get, rm, list operations and lifecycle subcommands.

mod add;
mod diff;
mod export;
mod import;
mod lock;
mod rotate;
mod unlock;

use tracing::info;

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

// Re-export subcommand functions
pub use add::execute as add;
pub use diff::execute as diff;
pub use export::execute as export;
pub use import::execute as import;
pub use lock::execute as lock;
pub use rotate::execute as rotate;
pub use unlock::execute as unlock;

/// Set a secret value.
pub fn set(key: &str, value: &str, force: bool) -> Result<()> {
    info!("Setting secret: {} (force: {})", key, force);
    let sp = output::spinner("encrypting...");
    let mut vault = Vault::open()?;
    vault.set(key, value, force)?;
    output::spinner_success(&sp, &format!("set {}", output::key(key)));
    Ok(())
}

/// Get a secret value.
pub fn get(key: &str) -> Result<()> {
    let vault = Vault::open()?;
    let value = vault.get(key)?;
    // Plain output for scripting - no decoration
    output::data(value.as_str());
    Ok(())
}

/// Remove a secret.
pub fn rm(key: &str) -> Result<()> {
    info!("Removing secret: {}", key);
    let mut vault = Vault::open()?;
    vault.remove(key)?;
    output::success(&format!("removed {}", output::key(key)));
    Ok(())
}

/// List all secret keys.
pub fn list(json: bool) -> Result<()> {
    let vault = Vault::open()?;
    let secrets = vault.list();

    if json {
        let keys: Vec<String> = secrets.iter().map(|s| s.key().to_string()).collect();
        let result = serde_json::json!({
            "keys": keys,
            "count": secrets.len()
        });
        output::data(&serde_json::to_string_pretty(&result)?);
    } else if secrets.is_empty() {
        output::dimmed("no secrets");
    } else {
        output::blank();
        output::header(&format!("{} secrets", output::count(secrets.len())));
        output::rule();
        for secret in secrets {
            output::list_item(secret.key());
        }
    }

    Ok(())
}
