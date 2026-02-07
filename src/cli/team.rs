//! Team management commands.
//!
//! Add, list, and remove team members (recipients).

use crate::cli::output;
use crate::core::config::Config;
use crate::core::team;
use crate::error::Result;

/// Add a team member.
pub fn add(name: &str, key: &str) -> Result<()> {
    let mut config = Config::load()?;
    team::add(&mut config, name, key)?;
    output::success(&format!("team member added: {}", output::key(name)));
    if !config.secrets.is_empty() {
        output::kv(
            "re-encrypted",
            format!("{} secrets for new recipient set", config.secrets.len()),
        );
    }
    Ok(())
}

/// List team members.
pub fn list(json: bool) -> Result<()> {
    let config = Config::load()?;
    let members = team::list(&config);

    if json {
        let members_json: Vec<_> = members
            .iter()
            .map(|(name, key)| {
                serde_json::json!({
                    "name": name,
                    "public_key": key
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
        for (name, key) in members {
            output::kv(&name, format!("{}...", &key[..24]));
        }
    }

    Ok(())
}

/// Remove a team member.
pub fn rm(name: &str) -> Result<()> {
    let mut config = Config::load()?;
    team::remove(&mut config, name)?;
    output::success(&format!("team member removed: {}", output::key(name)));
    Ok(())
}
