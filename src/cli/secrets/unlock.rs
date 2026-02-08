//! Unlock command.
//!
//! Decrypts secrets to local .env file.

use crate::cli::output;
use crate::error::Result;
use std::time::Instant;

/// Unlock secrets to .env file.
pub fn execute() -> Result<()> {
    let start = Instant::now();
    let vault = crate::core::vault::Vault::open()?;
    let sp = output::spinner("decrypting...");
    let env = vault.unlock()?;
    sp.finish_and_clear();

    output::timed(
        &format!(
            "decrypted {} secrets to {}",
            output::count(env.len()),
            output::path(".env")
        ),
        start.elapsed(),
    );
    Ok(())
}
