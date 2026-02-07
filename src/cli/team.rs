//! Team management commands.
//!
//! Add, list, and remove team members (recipients).

use tracing::info;

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// Add a team member.
pub fn add(name: &str, key: &str) -> Result<()> {
    info!("Adding team member: {}", name);
    let mut vault = Vault::open()?;
    let secret_count = vault.list().len();
    vault.add_recipient(name, key)?;
    output::success(&format!("team member added: {}", output::key(name)));
    if secret_count > 0 {
        output::kv(
            "re-encrypted",
            format!("{} secrets for new recipient set", secret_count),
        );
    }
    Ok(())
}

/// List team members.
pub fn list(json: bool) -> Result<()> {
    let vault = Vault::open()?;
    let members = vault.recipients();

    if json {
        let members_json: Vec<_> = members
            .iter()
            .map(|r| {
                serde_json::json!({
                    "name": r.name(),
                    "public_key": r.public_key()
                })
            })
            .collect();

        let result = serde_json::json!({
            "members": members_json,
            "count": members.len()
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if members.is_empty() {
        output::dimmed("no team members");
    } else {
        println!();
        output::header(&format!("{} team members", members.len()));
        output::rule();
        for recipient in members {
            output::kv(
                recipient.name(),
                format!("{}...", &recipient.public_key()[..24]),
            );
        }
    }

    Ok(())
}

/// Remove a team member.
pub fn rm(name: &str) -> Result<()> {
    let mut vault = Vault::open()?;
    vault.remove_recipient(name)?;
    output::success(&format!("team member removed: {}", output::key(name)));
    Ok(())
}
