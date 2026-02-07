//! Team member management commands.

use colored::Colorize;

use crate::core::config::Config;
use crate::core::team;
use crate::error::Result;

/// Add a team member.
pub fn add(name: &str, key: &str) -> Result<()> {
    let mut config = Config::load()?;
    team::add(&mut config, name, key)?;
    println!("{} {} added to team", "team:".green().bold(), name);
    if !config.secrets.is_empty() {
        println!(
            "  re-encrypted {} secrets for new recipient set",
            config.secrets.len()
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

        let output = serde_json::json!({
            "members": members_json,
            "count": members.len()
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else if members.is_empty() {
        println!("{}", "no team members".dimmed());
    } else {
        println!("{} members:", members.len().to_string().green().bold());
        for (name, key) in members {
            println!("  {} ({}...)", name, &key[..24]);
        }
    }

    Ok(())
}

/// Remove a team member.
pub fn rm(name: &str) -> Result<()> {
    let mut config = Config::load()?;
    team::remove(&mut config, name)?;
    println!("{} {} removed from team", "team:".green().bold(), name);
    Ok(())
}
