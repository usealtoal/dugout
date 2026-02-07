//! Secret lifecycle management commands.
//!
//! Lock, unlock, import, export, diff, and rotate operations.

use tracing::info;

use crate::cli::output;
use crate::core::{cipher, config, store};
use crate::error::Result;
use std::fs;
use std::path::PathBuf;

/// Lock (status check - secrets are always encrypted).
pub fn lock() -> Result<()> {
    let vault = crate::core::vault::Vault::open()?;
    output::progress("Checking encryption");
    output::progress_done(true);
    output::success(&format!(
        "locked: {} secrets encrypted in {}",
        vault.list().len(),
        output::path(".burrow.toml")
    ));
    output::kv("status", "safe to commit");
    Ok(())
}

/// Unlock secrets to .env file.
pub fn unlock() -> Result<()> {
    let vault = crate::core::vault::Vault::open()?;
    output::progress("Decrypting secrets");
    let count = vault.unlock()?;
    output::progress_done(true);
    output::success(&format!(
        "unlocked: {} secrets written to {}",
        count,
        output::path(".env")
    ));
    Ok(())
}

/// Import secrets from a .env file.
pub fn import(path: &str) -> Result<()> {
    let mut vault = crate::core::vault::Vault::open()?;
    let imported = vault.import(path)?;
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
    let vault = crate::core::vault::Vault::open()?;
    let result = vault.export()?;
    print!("{}", result);
    Ok(())
}

/// Show diff/status between encrypted and local .env.
pub fn diff() -> Result<()> {
    let vault = crate::core::vault::Vault::open()?;

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
    let toml_keys: std::collections::HashSet<_> = vault.list().into_iter().collect();

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
            output::cmd("burrow secrets unlock")
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
            output::cmd("burrow secrets import .env")
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
                output::cmd("burrow secrets unlock")
            ));
        }
    }

    Ok(())
}

/// Get the key archive directory.
fn archive_dir(project_id: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        crate::error::StoreError::GenerationFailed("unable to determine home directory".to_string())
    })?;
    Ok(home
        .join(".burrow")
        .join("keys")
        .join(project_id)
        .join("archive"))
}

/// Archive the old identity key with timestamp.
fn archive_old_key(project_id: &str) -> Result<()> {
    let old_key_path = dirs::home_dir()
        .ok_or_else(|| {
            crate::error::StoreError::GenerationFailed(
                "unable to determine home directory".to_string(),
            )
        })?
        .join(".burrow")
        .join("keys")
        .join(project_id)
        .join("identity.key");

    if !old_key_path.exists() {
        // No old key to archive
        return Ok(());
    }

    let archive_path = archive_dir(project_id)?;
    fs::create_dir_all(&archive_path)?;

    // Create timestamped archive filename
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let archive_file = archive_path.join(format!("identity.key.{}", timestamp));

    // Move old key to archive
    fs::rename(&old_key_path, &archive_file)?;

    output::dimmed(&format!("  archived old key: {}", archive_file.display()));

    Ok(())
}

/// Execute key rotation.
///
/// Performs the following steps:
/// 1. Decrypts all secrets with the current key
/// 2. Archives the old key with a timestamp
/// 3. Generates a new keypair
/// 4. Re-encrypts all secrets with the new key and existing recipients
/// 5. Updates configuration with the new public key
pub fn rotate() -> Result<()> {
    info!("Starting key rotation");
    output::section("Key Rotation");

    // Load config
    let mut cfg = config::Config::load()?;
    let project_id = cfg.project_id();

    // Check that we have a key to rotate
    if !store::has_key(&project_id) {
        return Err(crate::error::StoreError::NoPrivateKey(project_id.clone()).into());
    }

    // Step 1: Decrypt all secrets
    output::progress("Decrypting secrets");
    let identity = store::load_identity(&project_id)?;

    let mut decrypted_secrets = Vec::new();
    for (key, ciphertext) in &cfg.secrets {
        let plaintext = cipher::decrypt(ciphertext, &identity)?;
        decrypted_secrets.push((key.clone(), plaintext));
    }
    output::progress_done(true);
    output::dimmed(&format!(
        "  decrypted {} secret(s)",
        decrypted_secrets.len()
    ));

    // Step 2: Archive old key
    output::progress("Archiving old key");
    archive_old_key(&project_id)?;
    output::progress_done(true);

    // Step 3: Generate new keypair
    output::progress("Generating new keypair");
    let new_public_key = store::generate_keypair(&project_id)?;
    output::progress_done(true);
    output::dimmed(&format!("  new public key: {}", new_public_key));

    // Step 4: Get all recipient public keys (including new one)
    let mut recipients = Vec::new();

    // Add new key as a recipient
    recipients.push(cipher::parse_recipient(&new_public_key)?);

    // Add existing recipients (except if it's the old version of the project key)
    for key in cfg.recipients.values() {
        // Skip if it's the old project key (we already added the new one)
        if key != &new_public_key {
            recipients.push(cipher::parse_recipient(key)?);
        }
    }

    // Step 5: Re-encrypt all secrets
    output::progress("Re-encrypting secrets");
    cfg.secrets.clear();
    for (key, plaintext) in decrypted_secrets {
        let ciphertext = cipher::encrypt(&plaintext, &recipients)?;
        cfg.secrets.insert(key, ciphertext);
    }
    output::progress_done(true);

    // Step 6: Update config with new public key for the project owner
    // Find the project owner recipient (the one with the old key) and update it
    let owner_name = cfg
        .recipients
        .iter()
        .find(|(_, key)| *key != &new_public_key)
        .map(|(name, _)| name.clone())
        .or_else(|| cfg.recipients.keys().next().cloned());

    if let Some(name) = owner_name {
        cfg.recipients.insert(name, new_public_key.clone());
    }

    // Save updated config
    output::progress("Saving configuration");
    cfg.save()?;
    output::progress_done(true);

    println!();
    output::success("key rotation complete");
    output::hint("all secrets have been re-encrypted with the new key");

    Ok(())
}
