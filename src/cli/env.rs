//! .env file import/export/diff commands.

use crate::cli::output;
use crate::core::config::Config;
use crate::core::env;
use crate::error::Result;

/// Import secrets from a .env file.
pub fn import(path: &str) -> Result<()> {
    let mut config = Config::load()?;
    let imported = env::import(&mut config, path)?;
    output::success(&format!(
        "imported {} secrets from {}",
        imported.len(),
        output::path(path)
    ));
    for key in &imported {
        output::list_item(key);
    }
    Ok(())
}

/// Export secrets as .env format to stdout.
pub fn export() -> Result<()> {
    let config = Config::load()?;
    let result = env::export(&config)?;
    print!("{}", result);
    Ok(())
}

/// Show diff/status between encrypted and local .env.
pub fn diff() -> Result<()> {
    let config = Config::load()?;
    println!();
    output::header("Status");
    output::rule();
    output::kv(
        ".burrow.toml",
        format!("{} secrets (encrypted)", config.secrets.len()),
    );

    let env_path = std::path::Path::new(".env");
    if env_path.exists() {
        let env_content = std::fs::read_to_string(env_path)?;
        let env_count = env_content
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .count();
        output::kv(".env", format!("{} entries (plaintext)", env_count));
    } else {
        output::kv(".env", "not found");
        println!();
        output::hint(&format!(
            "Run {} to create .env file",
            output::cmd("burrow unlock")
        ));
    }

    Ok(())
}
