//! Shared CLI output helpers for consistent, beautiful terminal output.
//!
//! Color scheme (respects NO_COLOR):
//! - Green: success, checkmarks
//! - Red: errors
//! - Yellow: warnings
//! - Cyan: paths, commands, keys, hints
//! - Bold: headers, important values
//! - Dimmed: secondary info

use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use std::fmt::Display;
use std::time::Duration;

const RULE_WIDTH: usize = 56;

/// Terminal reference for width detection.
fn term_width() -> usize {
    Term::stdout().size().1 as usize
}

/// Print a success message with checkmark (green).
///
/// Example: `✓ initialized`
pub fn success(msg: &str) {
    println!("{} {}", style("✓").green(), msg);
}

/// Print an error message to stderr (red).
///
/// Example: `✗ file not found`
pub fn error(msg: &str) {
    eprintln!("{} {}", style("✗").red(), msg);
}

/// Print a warning message (yellow).
///
/// Example: `⚠ key already exists`
pub fn warn(msg: &str) {
    println!("{} {}", style("⚠").yellow(), msg);
}

/// Print a hint message (cyan).
///
/// Example: `→ run burrow unlock to decrypt`
pub fn hint(msg: &str) {
    println!("{} {}", style("→").cyan(), style(msg).cyan());
}

/// Print a bold section header.
///
/// Example: `Configuration`
pub fn header(title: &str) {
    println!("{}", style(title).bold());
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

/// Print a horizontal rule separator.
///
/// Example: `────────────────────────────────────────────────────────`
pub fn rule() {
    println!("{}", style("─".repeat(RULE_WIDTH)).dim());
}

/// Print a key-value pair (label dimmed, value bold).
///
/// Example: `  recipient:  alice`
pub fn kv(label: &str, value: impl Display) {
    println!("  {}  {}", style(label).dim(), style(value).bold());
}

/// Print a list item with bullet.
///
/// Example: `  • DATABASE_URL`
pub fn list_item(item: &str) {
    println!("  • {}", item);
}

/// Print a dimmed/secondary message.
///
/// Example: `no secrets stored`
pub fn dimmed(msg: &str) {
    println!("{}", style(msg).dim());
}

/// Print a plain note message.
///
/// Example: `This is a note`
pub fn note(msg: &str) {
    println!("{}", msg);
}

/// Print a blank line.
pub fn blank() {
    println!();
}

/// Print raw data with no decoration (for piping/scripting).
pub fn raw(data: &str) {
    print!("{}", data);
}

/// Print raw data line (for get command, JSON output, etc).
pub fn data(data: &str) {
    println!("{}", data);
}

/// Format a path string in cyan.
///
/// Returns a colored string that can be used inline.
pub fn path(p: &str) -> String {
    style(p).cyan().to_string()
}

/// Format a command string in green.
///
/// Returns a colored string that can be used inline.
pub fn cmd(c: &str) -> String {
    style(c).green().to_string()
}

/// Format a key name in cyan.
///
/// Returns a colored string that can be used inline.
pub fn key(k: &str) -> String {
    style(k).cyan().to_string()
}

/// Format a count/number in bold.
///
/// Returns a styled string that can be used inline.
pub fn count(n: impl Display) -> String {
    style(n).bold().to_string()
}

/// Start a spinner with a message. Returns the ProgressBar handle.
///
/// Call `spinner_success()` or `spinner_error()` to finish.
pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();

    // If not a TTY (e.g., in tests), use hidden style
    if !Term::stdout().is_term() {
        pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());
    } else {
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.enable_steady_tick(Duration::from_millis(80));
    }
    pb.set_message(msg.to_string());
    pb
}

/// Finish a spinner with success.
pub fn spinner_success(pb: &ProgressBar, msg: &str) {
    if pb.is_hidden() {
        // In non-TTY environments (like tests), print directly
        println!("{} {}", style("✓").green(), msg);
    } else {
        pb.set_style(ProgressStyle::with_template("{msg}").unwrap());
        pb.finish_with_message(format!("{} {}", style("✓").green(), msg));
    }
}

/// Finish a spinner with failure.
pub fn spinner_error(pb: &ProgressBar, msg: &str) {
    if pb.is_hidden() {
        // In non-TTY environments (like tests), print directly to stderr
        eprintln!("{} {}", style("✗").red(), msg);
    } else {
        pb.set_style(ProgressStyle::with_template("{msg}").unwrap());
        pb.finish_with_message(format!("{} {}", style("✗").red(), msg));
    }
}

/// Create a progress bar for batch operations.
pub fn progress_bar(total: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);

    // If not a TTY (e.g., in tests), use hidden style
    if !Term::stdout().is_term() {
        pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());
    } else {
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.cyan} {msg} [{bar:30.cyan/dim}] {pos}/{len}")
                .unwrap()
                .progress_chars("█▓░"),
        );
        pb.enable_steady_tick(Duration::from_millis(80));
    }
    pb.set_message(msg.to_string());
    pb
}

/// Print a timed success message.
pub fn timed(msg: &str, duration: Duration) {
    let ms = duration.as_millis();
    let time_str = if ms < 1000 {
        format!("{ms}ms")
    } else {
        format!("{:.1}s", duration.as_secs_f64())
    };
    println!("{} {} {}", style("✓").green(), msg, style(time_str).dim());
}

/// Print a formatted error box with suggestion.
pub fn error_box(title: &str, detail: &str, suggestion: Option<&str>) {
    let _term_w = term_width().min(72);
    eprintln!();
    eprintln!("{}", style(format!("  {} {}", "✗", title)).red().bold());
    eprintln!("  {}", detail);
    if let Some(sug) = suggestion {
        eprintln!();
        eprintln!("  {} {}", style("hint:").cyan().bold(), sug);
    }
    eprintln!();
}
