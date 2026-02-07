//! Team management commands.
//!
//! Add, list, and remove team members (recipients).

mod add;
mod list;
mod rm;

// Re-export command functions
pub use add::execute as add;
pub use list::execute as list;
pub use rm::execute as rm;
