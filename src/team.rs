use crate::config::BurrowConfig;
use crate::crypto;
use crate::error::{BurrowError, Result};
use crate::secrets;

/// Add a team member by name and public key
pub fn add_member(config: &mut BurrowConfig, name: &str, public_key: &str) -> Result<()> {
    // Validate the key first
    crypto::parse_recipient(public_key)?;

    config
        .recipients
        .insert(name.to_string(), public_key.to_string());
    config.save()?;

    // Re-encrypt all secrets for the new recipient set
    if !config.secrets.is_empty() {
        secrets::reencrypt_all(config)?;
    }

    Ok(())
}

/// Remove a team member
pub fn remove_member(config: &mut BurrowConfig, name: &str) -> Result<()> {
    if config.recipients.remove(name).is_none() {
        return Err(BurrowError::RecipientNotFound(name.to_string()));
    }

    config.save()?;

    // Re-encrypt all secrets without the removed recipient
    if !config.secrets.is_empty() {
        secrets::reencrypt_all(config)?;
    }

    Ok(())
}

/// List all team members
pub fn list_members(config: &BurrowConfig) -> Vec<(String, String)> {
    config
        .recipients
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}
