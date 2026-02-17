//! Whoami command - print your public key.

use crate::cli::output;
use crate::core::domain::Identity;
use crate::core::store;
use crate::error::Result;

/// Print your public key.
pub fn execute() -> Result<()> {
    if !store::has_global()? {
        output::error("no identity found");
        output::hint("run: dugout setup");
        return Err(
            crate::error::StoreError::NoPrivateKey("~/.dugout/identity".to_string()).into(),
        );
    }

    let pubkey = Identity::load_global_pubkey()?;
    output::data(&pubkey);

    Ok(())
}
