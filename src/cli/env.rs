//! .env file operations.
//!
//! Import, export, and diff operations for .env file integration.

use crate::cli::output;
use crate::core::config::Config;
use crate::core::env;
use crate::error::Result;

/// Import secrets from a .env file.
pub fn import(path: &str) -> Result<()> {
    let mut config = Config::load()?;
    let imported = env::import(&mut config, path)?;
    output::success(&format!(
        "imported {} secrets from {}",
        imported.len(),
        output::path(path)
    ));
    for key in &imported {
        output::list_item(key);
    }
    Ok(())
}

/// Export secrets as .env format to stdout.
pub fn export() -> Result<()> {
    let config = Config::load()?;
    let result = env::export(&config)?;
    print!("{}", result);
    Ok(())
}

/// Show diff/status between encrypted and local .env.
pub fn diff() -> Result<()> {
    let config = Config::load()?;

    // Parse .env file if it exists
    let mut env_keys = std::collections::HashSet::new();
    let env_path = std::path::Path::new(".env");
    if env_path.exists() {
        let env_content = std::fs::read_to_string(env_path)?;
        for line in env_content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                if let Some((key, _)) = line.split_once('=') {
                    env_keys.insert(key.trim().to_string());
                }
            }
        }
    }

    // Get keys from .burrow.toml
    let toml_keys: std::collections::HashSet<_> = config.secrets.keys().cloned().collect();

    // Calculate differences
    let synced: Vec<_> = toml_keys.intersection(&env_keys).collect();
    let missing_from_env: Vec<_> = toml_keys.difference(&env_keys).collect();
    let untracked: Vec<_> = env_keys.difference(&toml_keys).collect();

    output::section("Diff");

    // Synced keys
    if !synced.is_empty() {
        output::success("synced:");
        for key in &synced {
            println!("  {}", output::key(key));
        }
        println!();
    }

    // Missing from .env
    if !missing_from_env.is_empty() {
        output::warn("in .burrow.toml but not in .env:");
        for key in &missing_from_env {
            println!("  {}", output::key(key));
        }
        println!();
        output::hint(&format!(
            "Run {} to sync these secrets",
            output::cmd("burrow unlock")
        ));
    }

    // Untracked in .env
    if !untracked.is_empty() {
        output::warn("in .env but not tracked:");
        for key in &untracked {
            println!("  {}", output::key(key));
        }
        println!();
        output::hint(&format!(
            "Use {} to encrypt untracked secrets",
            output::cmd("burrow import .env")
        ));
    }

    // Summary
    if synced.is_empty() && missing_from_env.is_empty() && untracked.is_empty() {
        if env_path.exists() {
            output::success("All secrets in sync");
        } else {
            output::warn(".env file not found");
            println!();
            output::hint(&format!(
                "Run {} to create .env file",
                output::cmd("burrow unlock")
            ));
        }
    }

    Ok(())
}
