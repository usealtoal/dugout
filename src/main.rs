//! Burrow - An extremely fast secrets manager for developers.

use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use burrow::cli::output;
use burrow::cli::{execute, Cli};

fn main() {
    let cli = Cli::parse();

    // Initialize tracing subscriber with env-filter support
    let filter = EnvFilter::try_from_env("BURROW_LOG").unwrap_or_else(|_| {
        if cli.verbose {
            EnvFilter::new("burrow=debug")
        } else {
            EnvFilter::new("burrow=warn")
        }
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false).without_time())
        .init();

    if let Err(e) = execute(cli.command) {
        // Format error with suggestion if available
        let error_msg = e.to_string();
        let suggestion = match &e {
            burrow::error::Error::Config(burrow::error::ConfigError::NotInitialized) => {
                Some("run: burrow init")
            }
            burrow::error::Error::Store(burrow::error::StoreError::NoPrivateKey(_)) => {
                Some("run: burrow init")
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
