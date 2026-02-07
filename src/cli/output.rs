//! Shared CLI output helpers for consistent, beautiful terminal output.
//!
//! Color scheme (respects NO_COLOR):
//! - Green: success, checkmarks
//! - Red: errors
//! - Yellow: warnings
//! - Cyan: paths, commands, keys, hints
//! - Bold: headers, important values
//! - Dimmed: secondary info

use colored::Colorize;
use std::fmt::Display;
use std::io::{self, Write as IoWrite};

const RULE_WIDTH: usize = 56;

/// Check if color output is disabled via NO_COLOR env var.
fn colors_enabled() -> bool {
    std::env::var("NO_COLOR").is_err()
}

/// Print a success message with checkmark (green).
///
/// Example: `✓ initialized`
pub fn success(msg: &str) {
    if colors_enabled() {
        println!("{} {}", "✓".green(), msg);
    } else {
        println!("✓ {}", msg);
    }
}

/// Print an error message to stderr (red).
///
/// Example: `✗ file not found`
pub fn error(msg: &str) {
    if colors_enabled() {
        eprintln!("{} {}", "✗".red(), msg);
    } else {
        eprintln!("✗ {}", msg);
    }
}

/// Print a warning message (yellow).
///
/// Example: `⚠ key already exists`
pub fn warn(msg: &str) {
    if colors_enabled() {
        println!("{} {}", "⚠".yellow(), msg);
    } else {
        println!("⚠ {}", msg);
    }
}

/// Print a hint message (cyan).
///
/// Example: `→ run burrow unlock to decrypt`
pub fn hint(msg: &str) {
    if colors_enabled() {
        println!("{} {}", "→".cyan(), msg.cyan());
    } else {
        println!("→ {}", msg);
    }
}

/// Print a bold section header.
///
/// Example: `Configuration`
pub fn header(title: &str) {
    if colors_enabled() {
        println!("{}", title.bold());
    } else {
        println!("{}", title);
    }
}

/// Print a key-value pair (label dimmed, value bold).
///
/// Example: `  recipient:  alice`
pub fn kv(label: &str, value: impl Display) {
    if colors_enabled() {
        println!("  {}  {}", label.dimmed(), value.to_string().bold());
    } else {
        println!("  {}  {}", label, value);
    }
}

/// Print a list item with bullet.
///
/// Example: `  • DATABASE_URL`
pub fn list_item(item: &str) {
    println!("  • {}", item);
}

/// Print a horizontal rule separator.
///
/// Example: `────────────────────────────────────────────────────────`
pub fn rule() {
    if colors_enabled() {
        println!("{}", "─".repeat(RULE_WIDTH).dimmed());
    } else {
        println!("{}", "─".repeat(RULE_WIDTH));
    }
}

/// Format a path string in cyan.
///
/// Returns a colored string that can be used inline.
pub fn path(p: &str) -> String {
    if colors_enabled() {
        p.cyan().to_string()
    } else {
        p.to_string()
    }
}

/// Format a command string in green.
///
/// Returns a colored string that can be used inline.
pub fn cmd(c: &str) -> String {
    if colors_enabled() {
        c.green().to_string()
    } else {
        c.to_string()
    }
}

/// Format a key name in cyan.
///
/// Returns a colored string that can be used inline.
pub fn key(k: &str) -> String {
    if colors_enabled() {
        k.cyan().to_string()
    } else {
        k.to_string()
    }
}

/// Start a progress line in the format `Label... `.
///
/// Call `progress_done()` to finish the line.
pub fn progress(label: &str) {
    if colors_enabled() {
        print!("{}... ", label.dimmed());
    } else {
        print!("{}... ", label);
    }
    let _ = io::stdout().flush();
}

/// Finish a progress line with success/failure indicator.
pub fn progress_done(success: bool) {
    if colors_enabled() {
        if success {
            println!("{}", "ok".green());
        } else {
            println!("{}", "failed".red());
        }
    } else {
        println!("{}", if success { "ok" } else { "failed" });
    }
}

/// Print a dimmed/secondary message.
///
/// Example: `no secrets stored`
pub fn dimmed(msg: &str) {
    if colors_enabled() {
        println!("{}", msg.dimmed());
    } else {
        println!("{}", msg);
    }
}

/// Print a section header with a separator line.
///
/// Example:
/// ```text
/// Secrets
/// ────────────────────────────────────────────────────────
/// ```
pub fn section(title: &str) {
    println!();
    header(title);
    rule();
}
