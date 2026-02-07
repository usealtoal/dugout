//! Command implementations.
//!
//! Handler functions for each CLI command.

use colored::Colorize;

use crate::cli::{Command, TeamAction};
use crate::core::{config, import_export, secrets, team};
use crate::core::config::BurrowConfig;
use crate::core::keystore::KeyStore;
use crate::error::Result;

/// Execute a command.
///
/// # Arguments
///
/// * `command` - Parsed command from CLI
///
/// # Errors
///
/// Returns error if the command execution fails.
pub fn execute(command: Command) -> Result<()> {
    match command {
        Command::Init { name, no_banner } => cmd_init(name, no_banner),
        Command::Set { key, value, force } => cmd_set(&key, &value, force),
        Command::Get { key } => cmd_get(&key),
        Command::Rm { key } => cmd_rm(&key),
        Command::List => cmd_list(),
        Command::Lock => cmd_lock(),
        Command::Unlock => cmd_unlock(),
        Command::Run { command } => cmd_run(&command),
        Command::Team { action } => match action {
            TeamAction::Add { name, key } => cmd_team_add(&name, &key),
            TeamAction::List => cmd_team_list(),
            TeamAction::Rm { name } => cmd_team_rm(&name),
        },
        Command::Import { path } => cmd_import(&path),
        Command::Export => cmd_export(),
        Command::Diff => cmd_diff(),
    }
}

/// Initialize burrow in the current directory.
fn cmd_init(name: Option<String>, no_banner: bool) -> Result<()> {
    if BurrowConfig::exists() {
        return Err(crate::error::ConfigError::AlreadyInitialized.into());
    }

    if !no_banner {
        crate::cli::banner::print_banner();
    }

    let name = name.unwrap_or_else(whoami::username);

    let mut config = BurrowConfig::new();
    let project_id = config.project_id();

    let public_key = KeyStore::generate_keypair(&project_id)?;
    config.recipients.insert(name.clone(), public_key.clone());
    config.save()?;

    config::ensure_gitignore()?;

    println!("{}", "burrow initialized".green().bold());
    println!("  recipient: {} ({})", name, &public_key[..20]);
    println!("  config:    .burrow.toml (commit this)");
    println!("  key:       ~/.burrow/keys/{}/", project_id);
    println!();
    println!("Next: {} to add secrets", "burrow set KEY VALUE".cyan());

    Ok(())
}

/// Set a secret value.
fn cmd_set(key: &str, value: &str, force: bool) -> Result<()> {
    let mut config = BurrowConfig::load()?;
    secrets::set_secret(&mut config, key, value, force)?;
    println!("{} {}", "set:".green().bold(), key);
    Ok(())
}

/// Get a secret value.
fn cmd_get(key: &str) -> Result<()> {
    let config = BurrowConfig::load()?;
    let value = secrets::get_secret(&config, key)?;
    println!("{}", value);
    Ok(())
}

/// Remove a secret.
fn cmd_rm(key: &str) -> Result<()> {
    let mut config = BurrowConfig::load()?;
    secrets::remove_secret(&mut config, key)?;
    println!("{} {}", "removed:".green().bold(), key);
    Ok(())
}

/// List all secret keys.
fn cmd_list() -> Result<()> {
    let config = BurrowConfig::load()?;
    let keys = secrets::list_secrets(&config);

    if keys.is_empty() {
        println!("{}", "no secrets stored".dimmed());
    } else {
        println!("{} secrets:", keys.len().to_string().green().bold());
        for key in keys {
            println!("  {}", key);
        }
    }

    Ok(())
}

/// Lock (status check - secrets are always encrypted).
fn cmd_lock() -> Result<()> {
    let config = BurrowConfig::load()?;
    println!(
        "{} {} secrets encrypted in .burrow.toml",
        "locked:".green().bold(),
        config.secrets.len()
    );
    println!("  safe to commit");
    Ok(())
}

/// Unlock secrets to .env file.
fn cmd_unlock() -> Result<()> {
    let config = BurrowConfig::load()?;
    let count = import_export::unlock_to_file(&config)?;
    println!(
        "{} {} secrets written to .env",
        "unlocked:".green().bold(),
        count
    );
    Ok(())
}

/// Run a command with secrets injected as environment variables.
fn cmd_run(command: &[String]) -> Result<()> {
    let config = BurrowConfig::load()?;
    let exit_code = run_with_secrets(&config, command)?;
    std::process::exit(exit_code);
}

/// Run a command with decrypted secrets as environment variables.
fn run_with_secrets(config: &BurrowConfig, command: &[String]) -> Result<i32> {
    if command.is_empty() {
        return Err(crate::error::Error::Other("no command specified".to_string()));
    }

    let pairs = secrets::decrypt_all(config)?;

    let mut cmd = std::process::Command::new(&command[0]);
    cmd.args(&command[1..]);

    for (key, value) in pairs {
        cmd.env(key, value);
    }

    let status = cmd.status()?;
    Ok(status.code().unwrap_or(1))
}

/// Add a team member.
fn cmd_team_add(name: &str, key: &str) -> Result<()> {
    let mut config = BurrowConfig::load()?;
    team::add_member(&mut config, name, key)?;
    println!("{} {} added to team", "team:".green().bold(), name);
    if !config.secrets.is_empty() {
        println!("  re-encrypted {} secrets for new recipient set", config.secrets.len());
    }
    Ok(())
}

/// List team members.
fn cmd_team_list() -> Result<()> {
    let config = BurrowConfig::load()?;
    let members = team::list_members(&config);

    if members.is_empty() {
        println!("{}", "no team members".dimmed());
    } else {
        println!("{} members:", members.len().to_string().green().bold());
        for (name, key) in members {
            println!("  {} ({}...)", name, &key[..24]);
        }
    }

    Ok(())
}

/// Remove a team member.
fn cmd_team_rm(name: &str) -> Result<()> {
    let mut config = BurrowConfig::load()?;
    team::remove_member(&mut config, name)?;
    println!("{} {} removed from team", "team:".green().bold(), name);
    Ok(())
}

/// Import secrets from a .env file.
fn cmd_import(path: &str) -> Result<()> {
    let mut config = BurrowConfig::load()?;
    let imported = import_export::import_env(&mut config, path)?;
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
fn cmd_export() -> Result<()> {
    let config = BurrowConfig::load()?;
    let output = import_export::export_env(&config)?;
    print!("{}", output);
    Ok(())
}

/// Show diff/status between encrypted and local .env.
fn cmd_diff() -> Result<()> {
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
