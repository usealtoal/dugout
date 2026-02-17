//! Migrate file-based identities to macOS Keychain
//!
//! This command migrates existing file-based age identities to the macOS Keychain
//! for enhanced security with hardware-backed storage and biometric authentication.

#![cfg(target_os = "macos")]

use std::path::{Path, PathBuf};

use crate::cli::output;
use crate::core::domain::identity::Identity;
use crate::core::store::keychain::Keychain;
use crate::error::{Result, StoreError};

/// Execute the Keychain migration
pub fn execute(delete: bool, force: bool) -> Result<()> {
    let keychain = Keychain::new()?;
    let mut migrated: Vec<(String, PathBuf)> = Vec::new();
    let mut failed: Vec<(String, String)> = Vec::new();

    output::hint("Migrating identities to macOS Keychain...");

    // Migrate global identity
    if let Ok(global_path) = Identity::global_path() {
        if global_path.exists() {
            match migrate_identity(&keychain, &global_path, "global", force) {
                Ok(_) => {
                    output::success("✓ Migrated global identity");
                    migrated.push(("global".to_string(), global_path));
                }
                Err(e) => {
                    let err_msg = format!("{}", e);
                    output::error(&format!("✗ Failed to migrate global identity: {}", err_msg));
                    failed.push(("global".to_string(), err_msg));
                }
            }
        }
    }

    // Migrate project identities
    if let Ok(keys_dir) = Identity::base_dir() {
        if keys_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&keys_dir) {
                for entry in entries.flatten() {
                    let project_id = entry.file_name().to_string_lossy().to_string();
                    let key_path = entry.path().join("identity.key");

                    if key_path.exists() {
                        match migrate_identity(&keychain, &key_path, &project_id, force) {
                            Ok(_) => {
                                output::success(&format!("✓ Migrated {}", project_id));
                                migrated.push((project_id, key_path));
                            }
                            Err(e) => {
                                let err_msg = format!("{}", e);
                                output::error(&format!(
                                    "✗ Failed to migrate {}: {}",
                                    project_id, err_msg
                                ));
                                failed.push((project_id, err_msg));
                            }
                        }
                    }
                }
            }
        }
    }

    // Summary
    println!();
    if !migrated.is_empty() {
        output::success(&format!(
            "Successfully migrated {} identit{} to Keychain",
            migrated.len(),
            if migrated.len() == 1 { "y" } else { "ies" }
        ));
    }

    if !failed.is_empty() {
        output::error(&format!(
            "Failed to migrate {} identit{}",
            failed.len(),
            if failed.len() == 1 { "y" } else { "ies" }
        ));
    }

    // Optionally delete files
    if delete && !migrated.is_empty() {
        println!();
        if force || confirm_deletion(&migrated)? {
            for (name, path) in &migrated {
                match std::fs::remove_file(path) {
                    Ok(_) => output::hint(&format!("Deleted file for {}", name)),
                    Err(e) => output::error(&format!("Failed to delete {}: {}", name, e)),
                }
            }
        } else {
            output::hint("Kept original files (no files deleted)");
        }
    }

    if failed.is_empty() {
        Ok(())
    } else {
        Err(StoreError::MigrationFailed(format!(
            "{} identit{} failed to migrate",
            failed.len(),
            if failed.len() == 1 { "y" } else { "ies" }
        ))
        .into())
    }
}

/// Migrate a single identity to Keychain
fn migrate_identity(keychain: &Keychain, path: &Path, account: &str, force: bool) -> Result<()> {
    // Read the identity file
    let contents = std::fs::read_to_string(path).map_err(StoreError::ReadFailed)?;

    // Parse and validate it's a valid age identity
    let identity: age::x25519::Identity = contents
        .trim()
        .parse()
        .map_err(|e: &str| StoreError::InvalidFormat(e.to_string()))?;

    // Store in Keychain
    keychain.store_identity(account, contents.trim(), force)?;

    // Verify by attempting to load it back
    match keychain.load_from_keychain(account) {
        Ok(loaded) => {
            if loaded.public_key() != identity.to_public().to_string() {
                return Err(StoreError::MigrationFailed(
                    "Keychain verification failed - public key mismatch".to_string(),
                )
                .into());
            }
        }
        Err(e) => {
            // Verification failed - this is an error
            return Err(StoreError::MigrationFailed(format!(
                "Failed to verify Keychain storage: {}",
                e
            ))
            .into());
        }
    }

    Ok(())
}

/// Confirm deletion with the user
fn confirm_deletion(items: &[(String, PathBuf)]) -> Result<bool> {
    use dialoguer::Confirm;

    println!();
    println!("The following files will be deleted:");
    for (name, path) in items {
        println!("  {} ({})", name, path.display());
    }
    println!();

    Confirm::new()
        .with_prompt(format!(
            "Delete {} file{}?",
            items.len(),
            if items.len() == 1 { "" } else { "s" }
        ))
        .default(false)
        .interact()
        .map_err(Into::into)
}
