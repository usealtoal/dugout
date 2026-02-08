//! Team list command.
//!
//! List all team members (recipients).

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
        output::dimmed("no team members");
    } else {
        output::blank();
        output::header(&format!("{} team members", output::count(members.len())));
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
