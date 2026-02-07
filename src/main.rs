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
        // Format error with suggestion if available
        let error_msg = e.to_string();
        let suggestion = match &e {
            burrow::error::Error::Config(burrow::error::ConfigError::NotInitialized) => {
                Some("Run 'burrow init' to initialize burrow in this directory")
            }
            burrow::error::Error::Store(burrow::error::StoreError::NoPrivateKey(_)) => {
                Some("Make sure you've run 'burrow init' first")
            }
            _ => None,
        };

        if suggestion.is_some() {
            output::error_box("Error", &error_msg, suggestion);
        } else {
            output::error(&error_msg);
        }
        std::process::exit(1);
    }
}
