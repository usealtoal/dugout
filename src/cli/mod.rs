//! Command-line interface.

pub mod add;
pub mod admit;
pub mod banner;
pub mod completions;
pub mod dot;
pub mod init;
pub mod knock;
pub mod output;
pub mod pending;
pub mod run;
pub mod secrets;
pub mod setup;
pub mod shell;
pub mod team;
pub mod whoami;

// Subcommand groups
pub mod check;

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
    /// Enable verbose logging output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

/// Top-level commands.
#[derive(Subcommand)]
pub enum Command {
    /// Generate global identity at ~/.burrow/identity
    Setup {
        /// Overwrite existing identity
        #[arg(short, long)]
        force: bool,
    },

    /// Print your public key
    Whoami,

    /// Initialize burrow in the current directory
    Init {
        /// Your name (used as recipient identifier)
        #[arg(short, long)]
        name: Option<String>,
        /// Skip ASCII art banner
        #[arg(long)]
        no_banner: bool,
        /// Cipher backend: age (default), aws-kms, gcp-kms, gpg
        #[arg(long, value_name = "TYPE")]
        cipher: Option<String>,
        /// AWS KMS key ID or ARN (required for --cipher aws-kms)
        #[arg(long, value_name = "ARN")]
        kms_key: Option<String>,
        /// GCP KMS resource name (required for --cipher gcp-kms)
        #[arg(long, value_name = "RESOURCE")]
        gcp_key: Option<String>,
    },

    /// Add a secret interactively with hidden input
    Add {
        /// Secret key (e.g., DATABASE_URL)
        key: String,
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

    /// Request access to a vault
    Knock {
        /// Your name (optional, will prompt if not provided)
        name: Option<String>,
    },

    /// List pending access requests
    Pending,

    /// Approve an access request
    Admit {
        /// Name of the person to admit
        name: String,
    },

    /// Auto-detect project and run with secrets
    #[command(name = ".")]
    Dot,

    /// Run a command with secrets injected as env vars
    Run {
        /// Command and arguments to run
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },

    /// Spawn a shell with secrets loaded as environment variables
    Env,

    /// Manage team members
    #[command(subcommand)]
    Team(TeamAction),

    /// Secret lifecycle operations (lock, unlock, import, export, diff, rotate)
    #[command(subcommand)]
    Secrets(SecretsCommand),

    /// Run diagnostic checks (status, audit)
    #[command(subcommand)]
    Check(CheckCommand),

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

/// Secrets lifecycle subcommands.
#[derive(Subcommand)]
pub enum SecretsCommand {
    /// Encrypt all secrets (verify encryption status)
    Lock,

    /// Decrypt secrets to local .env file
    Unlock,

    /// Import secrets from a .env file
    Import {
        /// Path to .env file
        path: String,
    },

    /// Export secrets as .env format
    Export,

    /// Show diff between .burrow.toml and .env
    Diff,

    /// Rotate the project keypair and re-encrypt all secrets
    Rotate,
}

/// Check/diagnostic subcommands.
#[derive(Subcommand)]
pub enum CheckCommand {
    /// Show quick status overview
    Status,

    /// Audit git history for leaked secrets
    Audit,
}

/// Execute a command.
pub fn execute(command: Command) -> crate::error::Result<()> {
    use Command::*;

    match command {
        Setup { force } => setup::execute(force),
        Whoami => whoami::execute(),
        Init {
            name,
            no_banner,
            cipher,
            kms_key,
            gcp_key,
        } => init::execute(name, no_banner, cipher, kms_key, gcp_key),
        Add { key } => add::execute(&key),
        Set { key, value, force } => secrets::set(&key, &value, force),
        Get { key } => secrets::get(&key),
        Rm { key } => secrets::rm(&key),
        List { json } => secrets::list(json),
        Knock { name } => knock::execute(name),
        Pending => pending::execute(),
        Admit { name } => admit::execute(&name),
        Dot => dot::execute(),
        Run { command } => run::execute(&command),
        Env => shell::execute(),
        Team(action) => match action {
            TeamAction::Add { name, key } => team::add(&name, &key),
            TeamAction::List { json } => team::list(json),
            TeamAction::Rm { name } => team::rm(&name),
        },
        Secrets(cmd) => match cmd {
            SecretsCommand::Lock => secrets::lock(),
            SecretsCommand::Unlock => secrets::unlock(),
            SecretsCommand::Import { path } => secrets::import(&path),
            SecretsCommand::Export => secrets::export(),
            SecretsCommand::Diff => secrets::diff(),
            SecretsCommand::Rotate => secrets::rotate(),
        },
        Check(cmd) => match cmd {
            CheckCommand::Status => check::status(),
            CheckCommand::Audit => check::audit(),
        },
        Completions { shell } => completions::execute(shell),
    }
}
