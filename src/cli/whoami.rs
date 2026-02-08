//! Whoami command.
//!
//! Prints your global public key.

use crate::cli::output;
use crate::core::domain::Identity;
use crate::error::Result;

/// Print your public key.
pub fn execute() -> Result<()> {
    if !Identity::has_global()? {
        output::blank();
        output::error("no global identity found");
        output::blank();
        output::hint(&format!("run {} first", output::cmd("burrow setup")));
        return Err(
            crate::error::StoreError::NoPrivateKey("~/.burrow/identity".to_string()).into(),
        );
    }

    let pubkey = Identity::load_global_pubkey()?;
    output::data(&pubkey);

    Ok(())
}
