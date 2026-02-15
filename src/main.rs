//! Dugout - An extremely fast secrets manager for developers.

use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use dugout::cli::output;
use dugout::cli::{execute, Cli};

fn main() {
    let cli = Cli::parse();

    // Initialize tracing subscriber with env-filter support
    let filter = EnvFilter::try_from_env("DUGOUT_LOG").unwrap_or_else(|_| {
        if cli.verbose {
            EnvFilter::new("dugout=debug")
        } else {
            EnvFilter::new("dugout=warn")
        }
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false).without_time())
        .init();

    if let Err(e) = execute(cli.command, cli.vault) {
        // Format error with suggestion if available
        let error_msg = e.to_string();
        let suggestion = match &e {
            dugout::error::Error::Config(dugout::error::ConfigError::NotInitialized) => {
                Some("run: dugout init")
            }
            dugout::error::Error::Store(dugout::error::StoreError::NoPrivateKey(_)) => {
                Some("run: dugout init")
            }
            dugout::error::Error::Config(dugout::error::ConfigError::AccessDenied) => {
                Some("run: dugout knock")
            }
            _ => None,
        };

        output::error(&error_msg);
        if let Some(hint) = suggestion {
            output::hint(hint);
        }
        std::process::exit(1);
    }
}
