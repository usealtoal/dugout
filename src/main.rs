//! Burrow - An extremely fast secrets manager for developers.

use clap::Parser;

use burrow::cli::output;
use burrow::cli::{execute, Cli};

fn main() {
    let cli = Cli::parse();

    // Initialize tracing subscriber based on verbose flag
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_target(false)
            .with_level(true)
            .init();
    }

    if let Err(e) = execute(cli.command) {
        output::error(&e.to_string());
        std::process::exit(1);
    }
}
