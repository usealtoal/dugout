//! Rotate command - rotate keypair and re-encrypt secrets.

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
        .join(".dugout")
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
        .join(".dugout")
        .join("keys")
        .join(project_id)
        .join("identity.key");

    if !old_key_path.exists() {
        return Ok(());
    }

    let archive_path = archive_dir(project_id)?;
    fs::create_dir_all(&archive_path)?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let archive_file = archive_path.join(format!("identity.key.{}", timestamp));

    fs::rename(&old_key_path, &archive_file)?;

    Ok(())
}

/// Execute key rotation.
pub fn execute() -> Result<()> {
    info!("Starting key rotation");

    // Load config
    let mut cfg = config::Config::load()?;
    let project_id = cfg.project_id();

    // Check that we have a key to rotate
    if !store::has_key(&project_id) {
        return Err(crate::error::StoreError::NoPrivateKey(project_id.clone()).into());
    }

    // Step 1: Decrypt all secrets
    let identity = store::load_identity(&project_id)?;

    let mut decrypted_secrets = Vec::new();
    for (key, ciphertext) in &cfg.secrets {
        let plaintext = cipher::decrypt(ciphertext, identity.as_age())?;
        decrypted_secrets.push((key.clone(), plaintext));
    }

    // Step 2: Archive old key
    archive_old_key(&project_id)?;

    // Step 3: Generate new keypair
    let new_public_key = store::generate_keypair(&project_id)?;

    // Step 4: Get all recipient public keys (including new one)
    let mut recipients = Vec::new();
    recipients.push(cipher::parse_recipient(&new_public_key)?);

    for key in cfg.recipients.values() {
        if key != &new_public_key {
            recipients.push(cipher::parse_recipient(key)?);
        }
    }

    // Step 5: Re-encrypt all secrets
    let secret_count = decrypted_secrets.len();
    cfg.secrets.clear();
    for (key, plaintext) in decrypted_secrets {
        let ciphertext = cipher::encrypt(&plaintext, &recipients)?;
        cfg.secrets.insert(key, ciphertext);
    }

    // Step 6: Update config with new public key
    let owner_name = cfg
        .recipients
        .iter()
        .find(|(_, key)| *key != &new_public_key)
        .map(|(name, _)| name.clone())
        .or_else(|| cfg.recipients.keys().next().cloned());

    if let Some(name) = owner_name {
        cfg.recipients.insert(name, new_public_key.clone());
    }

    cfg.save()?;

    output::success(&format!("rotated ({} secrets re-encrypted)", secret_count));

    Ok(())
}
