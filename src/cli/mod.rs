//! Command-line interface definitions.

pub mod args;
pub mod banner;
pub mod commands;

pub use args::{Cli, Command, TeamAction};
pub use commands::execute;
