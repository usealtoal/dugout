//! Burrow - An extremely fast secrets manager for developers.

use clap::Parser;
use colored::Colorize;

use burrow::cli::{Cli, execute};

fn main() {
    let cli = Cli::parse();

    if let Err(e) = execute(cli.command) {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}
