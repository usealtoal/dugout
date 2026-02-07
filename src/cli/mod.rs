//! Command-line interface.

pub mod audit;
pub mod banner;
pub mod completions;
pub mod env;
pub mod init;
pub mod lock;
pub mod output;
pub mod rotate;
pub mod run;
pub mod secrets;
pub mod shell;
pub mod status;
pub mod team;

use clap::{Parser, Subcommand};

/// Burrow - An extremely fast secrets manager for developers.
#[derive(Parser)]
#[command(
    name = "burrow",
    about = "An extremely fast secrets manager for developers",
    version,
    after_help = "Dig deep. Ship safe. 🐀"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Top-level commands.
#[derive(Subcommand)]
pub enum Command {
    /// Initialize burrow in the current directory
    Init {
        /// Your name (used as recipient identifier)
        #[arg(short, long)]
        name: Option<String>,
        /// Skip ASCII art banner
        #[arg(long)]
        no_banner: bool,
    },

    /// Set a secret value
    Set {
        /// Secret key (e.g., DATABASE_URL)
        key: String,
        /// Secret value
        value: String,
        /// Overwrite if exists
        #[arg(short, long)]
        force: bool,
    },

    /// Get a secret value
    Get {
        /// Secret key
        key: String,
    },

    /// Remove a secret
    Rm {
        /// Secret key
        key: String,
    },

    /// List all secret keys
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Encrypt all secrets (lock the burrow)
    Lock,

    /// Decrypt secrets to local .env file
    Unlock,

    /// Run a command with secrets injected as env vars
    Run {
        /// Command and arguments to run
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },

    /// Spawn a shell with secrets loaded as environment variables
    Env,

    /// Manage team members
    Team {
        #[command(subcommand)]
        action: TeamAction,
    },

    /// Import secrets from a .env file
    Import {
        /// Path to .env file
        path: String,
    },

    /// Export secrets as .env format
    Export,

    /// Show diff between .burrow.toml and .env
    Diff,

    /// Show quick status overview
    Status,

    /// Audit git history for leaked secrets
    Audit,

    /// Rotate the project keypair and re-encrypt all secrets
    Rotate,

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// Supported shells for completions.
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

/// Team subcommands.
#[derive(Subcommand)]
pub enum TeamAction {
    /// Add a team member by their public key
    Add {
        /// Member name
        name: String,
        /// age public key
        key: String,
    },

    /// List team members
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Remove a team member
    Rm {
        /// Member name
        name: String,
    },
}

/// Execute a command.
pub fn execute(command: Command) -> crate::error::Result<()> {
    use Command::*;

    match command {
        Init { name, no_banner } => init::execute(name, no_banner),
        Set { key, value, force } => secrets::set(&key, &value, force),
        Get { key } => secrets::get(&key),
        Rm { key } => secrets::rm(&key),
        List { json } => secrets::list(json),
        Lock => lock::lock(),
        Unlock => lock::unlock(),
        Run { command } => run::execute(&command),
        Env => shell::execute(),
        Team { action } => match action {
            TeamAction::Add { name, key } => team::add(&name, &key),
            TeamAction::List { json } => team::list(json),
            TeamAction::Rm { name } => team::rm(&name),
        },
        Import { path } => env::import(&path),
        Export => env::export(),
        Diff => env::diff(),
        Status => status::execute(),
        Audit => audit::execute(),
        Rotate => rotate::execute(),
        Completions { shell } => completions::execute(shell),
    }
}
