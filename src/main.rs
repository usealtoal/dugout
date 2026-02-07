mod cli;
mod config;
mod crypto;
mod error;
mod import_export;
mod keystore;
mod runner;
mod secrets;
mod team;

use clap::Parser;
use colored::Colorize;

use cli::{Cli, Command, TeamAction};
use config::BurrowConfig;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> error::Result<()> {
    match cli.command {
        Command::Init { name } => cmd_init(name),
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

fn cmd_init(name: Option<String>) -> error::Result<()> {
    if BurrowConfig::exists() {
        return Err(error::BurrowError::AlreadyInitialized);
    }

    let name = name.unwrap_or_else(|| {
        whoami::username()
    });

    let mut config = BurrowConfig::new();
    let project_id = config.project_id();

    let public_key = keystore::KeyStore::generate_keypair(&project_id)?;
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

fn cmd_set(key: &str, value: &str, force: bool) -> error::Result<()> {
    let mut config = BurrowConfig::load()?;
    secrets::set_secret(&mut config, key, value, force)?;
    println!("{} {}", "set:".green().bold(), key);
    Ok(())
}

fn cmd_get(key: &str) -> error::Result<()> {
    let config = BurrowConfig::load()?;
    let value = secrets::get_secret(&config, key)?;
    println!("{}", value);
    Ok(())
}

fn cmd_rm(key: &str) -> error::Result<()> {
    let mut config = BurrowConfig::load()?;
    secrets::remove_secret(&mut config, key)?;
    println!("{} {}", "removed:".green().bold(), key);
    Ok(())
}

fn cmd_list() -> error::Result<()> {
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

fn cmd_lock() -> error::Result<()> {
    let config = BurrowConfig::load()?;
    println!(
        "{} {} secrets encrypted in .burrow.toml",
        "locked:".green().bold(),
        config.secrets.len()
    );
    println!("  safe to commit");
    Ok(())
}

fn cmd_unlock() -> error::Result<()> {
    let config = BurrowConfig::load()?;
    let count = import_export::unlock_to_file(&config)?;
    println!(
        "{} {} secrets written to .env",
        "unlocked:".green().bold(),
        count
    );
    Ok(())
}

fn cmd_run(command: &[String]) -> error::Result<()> {
    let config = BurrowConfig::load()?;
    let exit_code = runner::run_with_secrets(&config, command)?;
    std::process::exit(exit_code);
}

fn cmd_team_add(name: &str, key: &str) -> error::Result<()> {
    let mut config = BurrowConfig::load()?;
    team::add_member(&mut config, name, key)?;
    println!("{} {} added to team", "team:".green().bold(), name);
    if !config.secrets.is_empty() {
        println!("  re-encrypted {} secrets for new recipient set", config.secrets.len());
    }
    Ok(())
}

fn cmd_team_list() -> error::Result<()> {
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

fn cmd_team_rm(name: &str) -> error::Result<()> {
    let mut config = BurrowConfig::load()?;
    team::remove_member(&mut config, name)?;
    println!("{} {} removed from team", "team:".green().bold(), name);
    Ok(())
}

fn cmd_import(path: &str) -> error::Result<()> {
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

fn cmd_export() -> error::Result<()> {
    let config = BurrowConfig::load()?;
    let output = import_export::export_env(&config)?;
    print!("{}", output);
    Ok(())
}

fn cmd_diff() -> error::Result<()> {
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
