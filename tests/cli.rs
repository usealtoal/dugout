//! CLI integration tests.

mod support;

#[path = "cli/add.rs"]
mod add;
#[path = "cli/check.rs"]
mod check;
#[path = "cli/dot.rs"]
mod dot;
#[path = "cli/errors.rs"]
mod errors;
#[path = "cli/init.rs"]
mod init;
#[path = "cli/knock.rs"]
mod knock;
#[path = "cli/run.rs"]
mod run;
#[path = "cli/secrets.rs"]
mod secrets;
#[path = "cli/setup.rs"]
mod setup;
#[path = "cli/team.rs"]
mod team;
