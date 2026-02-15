//! Vault list command - list all vaults in the repository.

use crate::cli::output;
use crate::core::vault::Vault;
use crate::error::Result;

/// List all vaults in the repository.
pub fn execute(json: bool) -> Result<()> {
    let vaults = Vault::list_vaults()?;

    if vaults.is_empty() {
        output::data("no vaults found");
        output::hint("run: dugout init");
        return Ok(());
    }

    if json {
        let json_output: Vec<_> = vaults
            .iter()
            .map(|v| {
                serde_json::json!({
                    "name": v.name,
                    "path": v.path.display().to_string(),
                    "secrets": v.secret_count,
                    "recipients": v.recipient_count,
                    "access": v.has_access,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Calculate column width based on longest vault name (min 7 for "default")
        let name_width = vaults
            .iter()
            .map(|v| v.name.len())
            .max()
            .unwrap_or(7)
            .max(7);

        println!(
            "{:<width$} {:>8} {:>11} {:>7}",
            "VAULT", "SECRETS", "RECIPIENTS", "ACCESS",
            width = name_width
        );
        for v in vaults {
            let access = if v.has_access { "yes" } else { "no" };
            println!(
                "{:<width$} {:>8} {:>11} {:>7}",
                v.name, v.secret_count, v.recipient_count, access,
                width = name_width
            );
        }
    }

    Ok(())
}
