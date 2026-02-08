//! Rotate command.
//!
//! Rotate the project keypair and re-encrypt all secrets.

use tracing::info;

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
/// Performs the following steps:
/// 1. Decrypts all secrets with the current key
/// 2. Archives the old key with a timestamp
/// 3. Generates a new keypair
/// 4. Re-encrypts all secrets with the new key and existing recipients
/// 5. Updates configuration with the new public key
pub fn execute() -> Result<()> {
    use std::time::Instant;

    info!("Starting key rotation");
    let start = Instant::now();

    // Load config
    let mut cfg = config::Config::load()?;
    let project_id = cfg.project_id();

    // Check that we have a key to rotate
    if !store::has_key(&project_id) {
        return Err(crate::error::StoreError::NoPrivateKey(project_id.clone()).into());
    }

    // Step 1: Decrypt all secrets
    let sp = output::spinner(&format!(
        "decrypting {} secrets...",
        output::count(cfg.secrets.len())
    ));
    let identity = store::load_identity(&project_id)?;

    let mut decrypted_secrets = Vec::new();
    for (key, ciphertext) in &cfg.secrets {
        let plaintext = cipher::decrypt(ciphertext, identity.as_age())?;
        decrypted_secrets.push((key.clone(), plaintext));
    }
    sp.finish_and_clear();

    // Step 2: Archive old key
    let sp = output::spinner("archiving old key...");
    archive_old_key(&project_id)?;
    sp.finish_and_clear();

    // Step 3: Generate new keypair
    let sp = output::spinner("generating new keypair...");
    let new_public_key = store::generate_keypair(&project_id)?;
    sp.finish_and_clear();

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
    let secret_count = decrypted_secrets.len();
    let pb = output::progress_bar(secret_count as u64, "re-encrypting secrets");
    cfg.secrets.clear();
    for (key, plaintext) in decrypted_secrets {
        let ciphertext = cipher::encrypt(&plaintext, &recipients)?;
        cfg.secrets.insert(key, ciphertext);
        pb.inc(1);
    }
    pb.finish_and_clear();

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
    cfg.save()?;

    output::timed(
        &format!(
            "rotated key, re-encrypted {} secrets",
            output::count(secret_count)
        ),
        start.elapsed(),
    );

    Ok(())
}
