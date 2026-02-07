//! Command-line interface definitions.

pub mod args;
pub mod banner;
pub mod commands;

pub use args::{Cli, Command, Shell, TeamAction};
pub use commands::execute;
