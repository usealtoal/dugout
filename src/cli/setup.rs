//! Setup command.
//!
//! Generates a global identity at `~/.burrow/identity`.

use crate::cli::output;
use crate::core::domain::Identity;
use crate::error::Result;

/// Generate global identity.
pub fn execute(force: bool) -> Result<()> {
    // Check if identity already exists
    if Identity::has_global()? && !force {
        let pubkey = Identity::load_global_pubkey()?;
        output::blank();
        output::warn("global identity already exists");
        output::blank();
        output::kv("public key", &pubkey);
        output::blank();
        output::hint("use --force to overwrite");
        return Ok(());
    }

    let sp = output::spinner("generating keypair...");
    let identity = if force && Identity::has_global()? {
        // Overwrite existing
        Identity::generate_global()?
    } else {
        Identity::generate_global()?
    };
    output::spinner_success(&sp, "global identity created");

    let pubkey = identity.public_key();

    output::blank();
    output::success("your burrow identity:");
    output::blank();
    output::data(&pubkey);
    output::blank();
    output::kv("private key", "~/.burrow/identity");
    output::kv("public key", "~/.burrow/identity.pub");
    output::blank();
    output::hint("share your public key with team admins to request access");

    Ok(())
}
