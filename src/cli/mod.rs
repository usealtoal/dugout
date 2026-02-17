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
pub mod resolve;
pub mod run;
pub mod secrets;
pub mod setup;
pub mod shell;
pub mod sync;
pub mod team;
pub mod whoami;

// Subcommand groups
pub mod check;
pub mod vault;

// Platform-specific commands
#[cfg(target_os = "macos")]
pub mod migrate_keychain;

#[cfg(target_os = "macos")]
pub mod reset_keychain;

use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Parser, Subcommand};

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default())
    .valid(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::Red.on_default().effects(Effects::BOLD))
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD));

/// Dugout - Git-native secrets manager for development teams.
#[derive(Parser)]
#[command(
    name = "dugout",
    about = "Git-native secrets manager for development teams",
    version,
    styles = STYLES
)]
pub struct Cli {
    /// Enable verbose logging output
    #[arg(short = 'v', long, global = true)]
    pub verbose: bool,

    /// Select vault (e.g., "dev", "prod"). Uses .dugout.toml by default.
    #[arg(long = "vault", global = true, env = "DUGOUT_VAULT")]
    pub vault: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

/// Top-level commands.
#[derive(Subcommand)]
pub enum Command {
    /// Generate global identity at ~/.dugout/identity
    Setup {
        /// Overwrite existing identity
        #[arg(short, long)]
        force: bool,
        /// Identity name (non-interactive)
        #[arg(short, long)]
        name: Option<String>,
        /// Write private key to path (use - for stdout)
        #[arg(short, long, value_name = "PATH")]
        output: Option<String>,
    },

    /// Print your public key
    Whoami,

    /// Initialize dugout in the current directory
    Init {
        /// Your name (used as recipient identifier)
        #[arg(short, long)]
        name: Option<String>,
        /// Skip ASCII art banner
        #[arg(long)]
        no_banner: bool,
        /// KMS key for hybrid encryption (auto-detects AWS/GCP from format)
        #[arg(long, value_name = "KEY")]
        kms: Option<String>,
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

    /// Re-encrypt secrets for the current recipient set
    Sync {
        /// Show what would change without doing it
        #[arg(long)]
        dry_run: bool,
        /// Force re-encryption even if already in sync
        #[arg(long)]
        force: bool,
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

    /// Vault management commands
    #[command(subcommand)]
    Vault(VaultCommand),

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Migrate file-based identities to macOS Keychain (macOS only)
    #[cfg(target_os = "macos")]
    MigrateKeychain {
        /// Delete files after successful migration
        #[arg(long)]
        delete: bool,
        /// Skip confirmation prompts
        #[arg(short, long)]
        force: bool,
    },

    /// Remove identities from macOS Keychain (macOS only)
    #[cfg(target_os = "macos")]
    ResetKeychain {
        /// Account name to remove (e.g., "global", "project-id"), or omit to specify --all
        account: Option<String>,
        /// Remove all dugout identities from Keychain
        #[arg(long)]
        all: bool,
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
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

    /// Show diff between .dugout.toml and .env
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

/// Vault management subcommands.
#[derive(Subcommand)]
pub enum VaultCommand {
    /// List all vaults in the repository
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// Execute a command with vault context.
pub fn execute(command: Command, vault: Option<String>) -> crate::error::Result<()> {
    use Command::*;

    match command {
        Setup {
            force,
            name,
            output,
        } => setup::execute(force, name, output),
        Whoami => whoami::execute(),
        Init {
            name,
            no_banner,
            kms,
        } => init::execute(name, no_banner, kms, vault),
        Add { key } => add::execute(&key, vault),
        Set { key, value, force } => secrets::set(&key, &value, force, vault),
        Get { key } => secrets::get(&key, vault),
        Rm { key } => secrets::rm(&key, vault),
        List { json } => secrets::list(json, vault),
        Knock { name } => knock::execute(name, vault),
        Pending => pending::execute(vault),
        Admit { name } => admit::execute(&name, vault),
        Sync { dry_run, force } => sync::execute(dry_run, force, vault),
        Dot => dot::execute(vault),
        Run { command: cmd } => run::execute(&cmd, vault),
        Env => shell::execute(vault),
        Team(action) => match action {
            TeamAction::Add { name, key } => team::add(&name, &key, vault),
            TeamAction::List { json } => team::list(json, vault),
            TeamAction::Rm { name } => team::rm(&name, vault),
        },
        Secrets(cmd) => match cmd {
            SecretsCommand::Lock => secrets::lock(vault),
            SecretsCommand::Unlock => secrets::unlock(vault),
            SecretsCommand::Import { path } => secrets::import(&path, vault),
            SecretsCommand::Export => secrets::export(vault),
            SecretsCommand::Diff => secrets::diff(vault),
            SecretsCommand::Rotate => secrets::rotate(vault),
        },
        Check(cmd) => match cmd {
            CheckCommand::Status => check::status(vault),
            CheckCommand::Audit => check::audit(),
        },
        Vault(cmd) => match cmd {
            VaultCommand::List { json } => vault::list::execute(json),
        },
        Completions { shell } => completions::execute(shell),
        #[cfg(target_os = "macos")]
        MigrateKeychain { delete, force } => migrate_keychain::execute(delete, force),
        #[cfg(target_os = "macos")]
        ResetKeychain {
            account,
            all,
            force,
        } => reset_keychain::execute(account, all, force),
    }
}
