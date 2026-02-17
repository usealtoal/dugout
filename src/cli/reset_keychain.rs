//! Remove identities from macOS Keychain
//!
//! This command removes dugout identities from the macOS Keychain.

use crate::cli::output;
use crate::core::domain::identity::Identity;
use crate::core::store::keychain::Keychain;
use crate::error::Result;

/// Execute the Keychain reset
pub fn execute(account: Option<String>, all: bool, force: bool) -> Result<()> {
    let keychain = Keychain::new()?;

    if all {
        reset_all(&keychain, force)
    } else if let Some(account) = account {
        reset_account(&keychain, &account, force)
    } else {
        output::error("Must specify either an account name or --all");
        std::process::exit(1);
    }
}

/// Reset a specific account
fn reset_account(keychain: &Keychain, account: &str, force: bool) -> Result<()> {
    if !force && !confirm_single(account)? {
        output::hint("Cancelled");
        return Ok(());
    }

    output::hint(&format!("Removing identity '{}' from Keychain...", account));

    match keychain.delete_identity(account) {
        Ok(_) => {
            output::success(&format!("✓ Removed identity '{}' from Keychain", account));
            Ok(())
        }
        Err(e) => {
            output::error(&format!("✗ Failed to remove '{}': {}", account, e));
            Err(e)
        }
    }
}

/// Reset all dugout identities
fn reset_all(keychain: &Keychain, force: bool) -> Result<()> {
    let mut accounts = Vec::new();

    // Add global identity if exists
    accounts.push("global".to_string());

    // Add all project identities
    if let Ok(keys_dir) = Identity::base_dir() {
        if keys_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&keys_dir) {
                for entry in entries.flatten() {
                    let project_id = entry.file_name().to_string_lossy().to_string();
                    accounts.push(project_id);
                }
            }
        }
    }

    if accounts.is_empty() {
        output::hint("No identities found to remove");
        return Ok(());
    }

    if !force && !confirm_all(&accounts)? {
        output::hint("Cancelled");
        return Ok(());
    }

    output::hint("Removing all dugout identities from Keychain...");

    let mut removed = Vec::new();
    let mut failed = Vec::new();

    for account in &accounts {
        match keychain.delete_identity(account) {
            Ok(_) => {
                output::success(&format!("✓ Removed '{}'", account));
                removed.push(account.clone());
            }
            Err(e) => {
                // Don't fail on "not found" errors - just skip
                if !format!("{}", e).contains("not found") {
                    output::error(&format!("✗ Failed to remove '{}': {}", account, e));
                    failed.push((account.clone(), format!("{}", e)));
                }
            }
        }
    }

    println!();
    if !removed.is_empty() {
        output::success(&format!(
            "Successfully removed {} identit{} from Keychain",
            removed.len(),
            if removed.len() == 1 { "y" } else { "ies" }
        ));
    }

    if !failed.is_empty() {
        output::error(&format!(
            "Failed to remove {} identit{}",
            failed.len(),
            if failed.len() == 1 { "y" } else { "ies" }
        ));
    }

    Ok(())
}

/// Confirm deletion of a single account
fn confirm_single(account: &str) -> Result<bool> {
    use dialoguer::Confirm;

    Confirm::new()
        .with_prompt(format!("Remove identity '{}' from Keychain?", account))
        .default(false)
        .interact()
        .map_err(Into::into)
}

/// Confirm deletion of all accounts
fn confirm_all(accounts: &[String]) -> Result<bool> {
    use dialoguer::Confirm;

    println!();
    println!("The following identities will be removed from Keychain:");
    for account in accounts {
        println!("  {}", account);
    }
    println!();

    Confirm::new()
        .with_prompt(format!(
            "Remove {} identit{} from Keychain?",
            accounts.len(),
            if accounts.len() == 1 { "y" } else { "ies" }
        ))
        .default(false)
        .interact()
        .map_err(Into::into)
}
