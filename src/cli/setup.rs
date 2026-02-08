//! Setup command - generate global identity.

use crate::cli::output;
use crate::core::domain::Identity;
use crate::error::Result;

/// Generate global identity.
pub fn execute(force: bool) -> Result<()> {
    // Check if identity already exists
    if Identity::has_global()? && !force {
        let pubkey = Identity::load_global_pubkey()?;
        output::warn("identity already exists");
        output::hint(&format!("public key: {}", pubkey));
        output::hint("use --force to overwrite");
        return Ok(());
    }

    let identity = Identity::generate_global()?;
    let pubkey = identity.public_key();

    output::success("generated identity");
    output::hint(&format!("public key: {}", pubkey));

    Ok(())
}
