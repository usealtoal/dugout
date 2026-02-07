use crate::config::BurrowConfig;
use crate::crypto;
use crate::error::{BurrowError, Result};
use crate::keystore::KeyStore;

use age::x25519;

/// Get all recipient public keys from config
fn get_recipients(config: &BurrowConfig) -> Result<Vec<x25519::Recipient>> {
    config
        .recipients
        .values()
        .map(|k| crypto::parse_recipient(k))
        .collect()
}

/// Set (encrypt) a secret
pub fn set_secret(config: &mut BurrowConfig, key: &str, value: &str, force: bool) -> Result<()> {
    if config.secrets.contains_key(key) && !force {
        return Err(BurrowError::SecretExists(key.to_string()));
    }

    let recipients = get_recipients(config)?;
    if recipients.is_empty() {
        return Err(BurrowError::Config(
            "no recipients configured, run `burrow init` first".to_string(),
        ));
    }

    let encrypted = crypto::encrypt(value, &recipients)?;
    config.secrets.insert(key.to_string(), encrypted);
    config.save()?;

    Ok(())
}

/// Get (decrypt) a secret
pub fn get_secret(config: &BurrowConfig, key: &str) -> Result<String> {
    let encrypted = config
        .secrets
        .get(key)
        .ok_or_else(|| BurrowError::SecretNotFound(key.to_string()))?;

    let identity = KeyStore::load_identity(&config.project_id())?;
    let plaintext = crypto::decrypt(encrypted, &identity)?;

    Ok(plaintext)
}

/// Remove a secret
pub fn remove_secret(config: &mut BurrowConfig, key: &str) -> Result<()> {
    if config.secrets.remove(key).is_none() {
        return Err(BurrowError::SecretNotFound(key.to_string()));
    }
    config.save()?;
    Ok(())
}

/// List all secret keys
pub fn list_secrets(config: &BurrowConfig) -> Vec<String> {
    config.secrets.keys().cloned().collect()
}

/// Decrypt all secrets (for unlock/run)
pub fn decrypt_all(config: &BurrowConfig) -> Result<Vec<(String, String)>> {
    let identity = KeyStore::load_identity(&config.project_id())?;

    let mut pairs = Vec::new();
    for (key, encrypted) in &config.secrets {
        let plaintext = crypto::decrypt(encrypted, &identity)?;
        pairs.push((key.clone(), plaintext));
    }

    Ok(pairs)
}

/// Re-encrypt all secrets (e.g., after adding a team member)
pub fn reencrypt_all(config: &mut BurrowConfig) -> Result<()> {
    let identity = KeyStore::load_identity(&config.project_id())?;
    let recipients = get_recipients(config)?;

    let mut updated = std::collections::BTreeMap::new();
    for (key, encrypted) in &config.secrets {
        let plaintext = crypto::decrypt(encrypted, &identity)?;
        let reencrypted = crypto::encrypt(&plaintext, &recipients)?;
        updated.insert(key.clone(), reencrypted);
    }

    config.secrets = updated;
    config.save()?;

    Ok(())
}
