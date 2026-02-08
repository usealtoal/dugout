//! Constants used throughout dugout.
//!
//! Centralizes magic strings and configuration values.

/// Configuration file name (.dugout.toml).
pub const CONFIG_FILE: &str = ".dugout.toml";

/// Environment variables file name (.env).
#[allow(dead_code)]
pub const ENV_FILE: &str = ".env";

/// Key storage directory relative to HOME (~/.dugout/keys).
pub const KEY_DIR: &str = ".dugout/keys";

/// Gitignore entries to protect secrets.
///
/// These entries ensure that .env files are not accidentally committed.
pub const GITIGNORE_ENTRIES: &[&str] = &[".env", ".env.*", "!.env.example"];
