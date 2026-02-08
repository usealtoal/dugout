//! Team list command - list all team members.

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// List team members.
pub fn execute(json: bool) -> Result<()> {
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
        output::data(&serde_json::to_string_pretty(&result)?);
    } else if members.is_empty() {
        output::data("no team members");
    } else {
        for recipient in members {
            let truncated = if recipient.public_key().len() > 20 {
                format!("{}...", &recipient.public_key()[..20])
            } else {
                recipient.public_key().to_string()
            };
            println!("{:<15} {}", recipient.name(), truncated);
        }
    }

    Ok(())
}
