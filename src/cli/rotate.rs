//! Key rotation command.
//!
//! Rotate the project keypair by generating a new key and re-encrypting all secrets.

use crate::cli::output;
use crate::core::{cipher, config, store};
use crate::error::Result;

use std::fs;
use std::path::PathBuf;

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
/// 1. Load current config and decrypt all secrets
/// 2. Generate new keypair
/// 3. Archive old key
/// 4. Re-encrypt all secrets with new key + existing recipients
/// 5. Update config with new recipient
pub fn execute() -> Result<()> {
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
