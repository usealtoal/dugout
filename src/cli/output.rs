//! Minimal CLI output helpers.
//!
//! Color scheme (respects NO_COLOR):
//! - Green: success ✓
//! - Red: errors ✗
//! - Yellow: warnings ⚠
//! - Cyan: paths, commands, keys
//! - Bold: emphasis
//! - Dim: hints

use console::style;
use std::fmt::Display;

/// Print a success message with checkmark (green).
///
/// Example: `✓ initialized vault`
pub fn success(msg: &str) {
    println!("{} {}", style("✓").green(), msg);
}

/// Print an error message to stderr (red).
///
/// Example: `✗ vault not initialized`
pub fn error(msg: &str) {
    eprintln!("{} {}", style("✗").red(), msg);
}

/// Print a warning message (yellow).
///
/// Example: `⚠ key already exists`
pub fn warn(msg: &str) {
    println!("{} {}", style("⚠").yellow(), msg);
}

/// Print a hint message (dim, for actionable suggestions after errors).
///
/// Example: `  run: dugout init`
pub fn hint(msg: &str) {
    println!("  {}", style(msg).dim());
}

/// Print a key-value pair (label: value).
///
/// Example: `vault: .dugout.toml`
pub fn kv(label: &str, value: impl Display) {
    println!("{}: {}", label, value);
}

/// Print a list item.
///
/// Example: `DATABASE_URL`
pub fn list_item(item: &str) {
    println!("{}", item);
}

/// Print raw data with no decoration (for piping/scripting).
pub fn raw(data: &str) {
    print!("{}", data);
}

/// Print data line (for get command, list output, etc).
pub fn data(data: &str) {
    println!("{}", data);
}

/// Format a path string in cyan.
pub fn path(p: &str) -> String {
    style(p).cyan().to_string()
}

/// Format a command string in cyan.
pub fn cmd(c: &str) -> String {
    style(c).cyan().to_string()
}

/// Format a key name in cyan.
pub fn key(k: &str) -> String {
    style(k).cyan().to_string()
}

/// Format a count/number in parentheses.
pub fn count(n: impl Display) -> String {
    format!("{}", n)
}
