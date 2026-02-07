use clap::{Parser, Subcommand};

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

#[derive(Subcommand)]
pub enum Command {
    /// Initialize burrow in the current directory
    Init {
        /// Your name (used as recipient identifier)
        #[arg(short, long)]
        name: Option<String>,
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
    List,

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

    /// Show diff since last lock
    Diff,
}

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
    List,

    /// Remove a team member
    Rm {
        /// Member name
        name: String,
    },
}
