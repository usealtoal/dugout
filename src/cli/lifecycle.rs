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
    let env = vault.unlock()?;
    output::progress_done(true);
    output::success(&format!(
        "unlocked: {} secrets written to {}",
        env.len(),
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
    let env = vault.export()?;
    print!("{}", env);
    Ok(())
}

/// Show diff/status between encrypted and local .env.
pub fn diff() -> Result<()> {
    let vault = crate::core::vault::Vault::open()?;
    let env_path = std::path::Path::new(".env");
    let diff = vault.diff(env_path)?;

    output::section("Diff");

    // Synced entries
    let synced = diff.synced();
    if !synced.is_empty() {
        output::success("synced:");
        for entry in &synced {
            println!("  {}", output::key(entry.key()));
        }
        println!();
    }

    // Modified entries
    let modified = diff.modified();
    if !modified.is_empty() {
        output::warn("modified (values differ):");
        for entry in &modified {
            println!("  {}", output::key(entry.key()));
        }
        println!();
        output::hint(&format!(
            "Run {} to update .env with vault values",
            output::cmd("burrow secrets unlock")
        ));
    }

    // Vault-only entries
    let vault_only = diff.vault_only();
    if !vault_only.is_empty() {
        output::warn("in vault but not in .env:");
        for entry in &vault_only {
            println!("  {}", output::key(entry.key()));
        }
        println!();
        output::hint(&format!(
            "Run {} to sync these secrets",
            output::cmd("burrow secrets unlock")
        ));
    }

    // Env-only entries
    let env_only = diff.env_only();
    if !env_only.is_empty() {
        output::warn("in .env but not tracked:");
        for entry in &env_only {
            println!("  {}", output::key(entry.key()));
        }
        println!();
        output::hint(&format!(
            "Use {} to encrypt untracked secrets",
            output::cmd("burrow secrets import .env")
        ));
    }

    // Summary
    if diff.is_empty() {
        if env_path.exists() {
            output::warn(".env is empty");
        } else {
            output::warn(".env file not found");
        }
        println!();
        output::hint(&format!(
            "Run {} to create .env file",
            output::cmd("burrow secrets unlock")
        ));
    } else if diff.is_synced() {
        output::success("All secrets in sync");
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
        let plaintext = cipher::decrypt(ciphertext, identity.as_age())?;
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
