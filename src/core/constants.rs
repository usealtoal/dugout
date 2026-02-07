//! Constants used throughout burrow.
//!
//! Centralizes magic strings and configuration values.

/// Configuration file name (.burrow.toml).
pub const CONFIG_FILE: &str = ".burrow.toml";

/// Environment variables file name (.env).
pub const ENV_FILE: &str = ".env";

/// Key storage directory relative to HOME (~/.burrow/keys).
pub const KEY_DIR: &str = ".burrow/keys";

/// Gitignore entries to protect secrets.
///
/// These entries ensure that .env files are not accidentally committed.
pub const GITIGNORE_ENTRIES: &[&str] = &[".env", ".env.*", "!.env.example"];
