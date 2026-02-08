//! Setup command - generate global identity.

use crate::cli::output;
use crate::core::domain::Identity;
use crate::error::Result;

/// Generate global identity.
pub fn execute(force: bool, _name: Option<String>, output_path: Option<String>) -> Result<()> {
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

    // Output private key if requested
    if let Some(ref path) = output_path {
        use age::secrecy::ExposeSecret;
        let secret = identity.as_age().to_string();
        let key_str = secret.expose_secret();

        if path == "-" {
            output::raw(key_str);
            eprintln!("public key: {}", pubkey);
            return Ok(());
        }

        std::fs::write(path, format!("{}\n", key_str))
            .map_err(crate::error::StoreError::WriteFailed)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
                .map_err(crate::error::StoreError::WriteFailed)?;
        }

        output::success(&format!("private key written to {}", path));
    }

    output::success("generated identity");
    output::hint(&format!("public key: {}", pubkey));

    Ok(())
}
