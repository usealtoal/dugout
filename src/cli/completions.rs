//! Shell completion generation.

use clap::CommandFactory;
use clap_complete::{generate, Shell as CompletionShell};

use crate::cli::{Cli, Shell};
use crate::error::Result;

/// Generate shell completions.
pub fn execute(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let shell = match shell {
        Shell::Bash => CompletionShell::Bash,
        Shell::Zsh => CompletionShell::Zsh,
        Shell::Fish => CompletionShell::Fish,
        Shell::PowerShell => CompletionShell::PowerShell,
    };

    generate(shell, &mut cmd, "burrow", &mut std::io::stdout());
    Ok(())
}
