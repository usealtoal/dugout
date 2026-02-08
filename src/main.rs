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
                Some("run 'burrow init' to get started")
            }
            burrow::error::Error::Store(burrow::error::StoreError::NoPrivateKey(_)) => {
                Some("run 'burrow init' or check your key directory")
            }
            _ => None,
        };

        if suggestion.is_some() {
            output::error_box("error", &error_msg, suggestion);
        } else {
            output::error(&error_msg);
        }
        std::process::exit(1);
    }
}
