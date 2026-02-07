//! Import command.
//!
//! Import secrets from a .env file.

use crate::cli::output;
use crate::error::Result;
use std::time::Instant;

/// Import secrets from a .env file.
pub fn execute(path: &str) -> Result<()> {
    let start = Instant::now();
    let mut vault = crate::core::vault::Vault::open()?;

    let sp = output::spinner(&format!("Importing from {}...", output::path(path)));
    let imported = vault.import(path)?;
    sp.finish_and_clear();

    output::timed(
        &format!(
            "Imported {} secrets from {}",
            output::count(imported.len()),
            output::path(path)
        ),
        start.elapsed(),
    );

    if !imported.is_empty() {
        output::blank();
        for key in &imported {
            output::list_item(key);
        }
    }
    Ok(())
}
