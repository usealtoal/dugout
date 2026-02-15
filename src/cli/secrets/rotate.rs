//! Rotate command - rotate keypair and re-encrypt secrets.

use tracing::info;

use crate::cli::output;
use crate::core::vault::Vault;
use crate::core::{cipher, config, store};
use crate::error::Result;
use std::fs;
use std::path::PathBuf;

/// Get the key archive directory.
fn archive_dir(project_id: &str) -> Result<PathBuf> {
    use crate::core::domain::Identity;
    let key_dir = Identity::project_dir(project_id)?;
    Ok(key_dir.join("archive"))
}

/// Archive the old identity key with timestamp.
///
/// Uses atomic rename and handles missing files gracefully to avoid TOCTOU races.
fn archive_old_key(project_id: &str) -> Result<()> {
    use crate::core::domain::Identity;
    let old_key_path = Identity::project_dir(project_id)?.join("identity.key");

    let archive_path = archive_dir(project_id)?;
    fs::create_dir_all(&archive_path)?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let archive_file = archive_path.join(format!("identity.key.{}", timestamp));

    // Try to rename directly - handles TOCTOU by checking error instead of exists()
    match fs::rename(&old_key_path, &archive_file) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // No key to archive
        Err(e) => Err(e.into()),
    }
}

/// Execute key rotation.
pub fn execute(vault: Option<String>) -> Result<()> {
    let vault_name = crate::cli::resolve::resolve_vault(vault.as_deref())?;
    info!("Starting key rotation");

    // Verify access and pick the effective identity for decryption.
    let v = Vault::open_vault(vault_name.as_deref())?;
    let old_public_key = v.identity().public_key();

    // Load config
    let mut cfg = config::Config::load_from(vault_name.as_deref())?;
    let project_id = cfg.project_id();
    let backend = cipher::CipherBackend::from_config(&cfg)?;

    // Check that we have a key to rotate
    if !store::has_key(&project_id) {
        return Err(crate::error::StoreError::NoPrivateKey(project_id.clone()).into());
    }

    // Step 1: Decrypt all secrets
    let mut decrypted_secrets = Vec::new();
    for (key, ciphertext) in &cfg.secrets {
        let plaintext = backend.decrypt(ciphertext, v.identity().as_age())?;
        decrypted_secrets.push((key.clone(), plaintext));
    }

    // Step 2: Archive old key
    archive_old_key(&project_id)?;

    // Step 3: Generate new keypair
    let new_public_key = store::generate_keypair(&project_id)?;

    // Step 4: Build recipient set with rotated owner key.
    let mut recipients: Vec<String> = cfg
        .recipients
        .values()
        .map(|key| {
            if key == &old_public_key {
                new_public_key.clone()
            } else {
                key.clone()
            }
        })
        .collect();
    if !recipients.iter().any(|key| key == &new_public_key) {
        recipients.push(new_public_key.clone());
    }
    recipients.sort();
    recipients.dedup();

    // Step 5: Re-encrypt all secrets
    let secret_count = decrypted_secrets.len();
    cfg.secrets.clear();
    for (key, plaintext) in decrypted_secrets {
        let ciphertext = backend.encrypt(&plaintext, &recipients)?;
        cfg.secrets.insert(key, ciphertext);
    }

    // Step 6: Update config with new public key
    let owner_name = cfg
        .recipients
        .iter()
        .find(|(_, key)| *key == &old_public_key)
        .map(|(name, _)| name.clone())
        .or_else(|| cfg.recipients.keys().next().cloned());

    if let Some(name) = owner_name {
        cfg.recipients.insert(name, new_public_key.clone());
    }

    cfg.save_to(vault_name.as_deref())?;

    output::success(&format!("rotated ({} secrets re-encrypted)", secret_count));

    Ok(())
}
