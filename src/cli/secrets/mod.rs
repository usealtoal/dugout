//! Secret management commands.

mod diff;
mod export;
mod import;
mod lock;
mod rotate;
mod unlock;

use tracing::debug;

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

// Re-export subcommand functions
pub use diff::execute as diff;
pub use export::execute as export;
pub use import::execute as import;
pub use lock::execute as lock;
pub use rotate::execute as rotate;
pub use unlock::execute as unlock;

/// Set a secret value.
pub fn set(key: &str, value: &str, force: bool, vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;
    debug!("Setting secret: {} (force: {})", key, force);
    let mut v = Vault::open_vault(vault_name.as_deref())?;
    v.set(key, value, force)?;
    output::success(&format!("set {}", key));
    Ok(())
}

/// Get a secret value.
pub fn get(key: &str, vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;
    let v = Vault::open_vault(vault_name.as_deref())?;
    let value = v.get(key)?;
    // Plain output for scripting - no decoration
    output::data(value.as_str());
    Ok(())
}

/// Remove a secret.
pub fn rm(key: &str, vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;
    debug!("Removing secret: {}", key);
    let mut v = Vault::open_vault(vault_name.as_deref())?;
    v.remove(key)?;
    output::success(&format!("removed {}", key));
    Ok(())
}

/// List all secret keys.
pub fn list(json: bool, vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;
    let v = Vault::open_vault(vault_name.as_deref())?;
    let secrets = v.list();

    if json {
        let keys: Vec<String> = secrets.iter().map(|s| s.key().to_string()).collect();
        let result = serde_json::json!({
            "keys": keys,
            "count": secrets.len()
        });
        output::data(&serde_json::to_string_pretty(&result)?);
    } else if secrets.is_empty() {
        output::data("no secrets");
    } else {
        for secret in secrets {
            output::list_item(secret.key());
        }
    }

    Ok(())
}
