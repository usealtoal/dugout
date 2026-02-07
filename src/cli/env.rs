//! .env file import/export/diff commands.

use colored::Colorize;

use crate::core::config::BurrowConfig;
use crate::core::env;
use crate::error::Result;

/// Import secrets from a .env file.
pub fn import(path: &str) -> Result<()> {
    let mut config = BurrowConfig::load()?;
    let imported = env::import_env(&mut config, path)?;
    println!(
        "{} {} secrets from {}",
        "imported:".green().bold(),
        imported.len(),
        path
    );
    for key in &imported {
        println!("  {}", key);
    }
    Ok(())
}

/// Export secrets as .env format to stdout.
pub fn export() -> Result<()> {
    let config = BurrowConfig::load()?;
    let output = env::export_env(&config)?;
    print!("{}", output);
    Ok(())
}

/// Show diff/status between encrypted and local .env.
pub fn diff() -> Result<()> {
    let config = BurrowConfig::load()?;
    println!(
        "{} {} secrets in .burrow.toml",
        "status:".green().bold(),
        config.secrets.len()
    );

    let env_path = std::path::Path::new(".env");
    if env_path.exists() {
        let env_content = std::fs::read_to_string(env_path)?;
        let env_count = env_content
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .count();
        println!("  .env has {} entries", env_count);
    } else {
        println!("  {} (run `burrow unlock`)", ".env not found".dimmed());
    }

    Ok(())
}
