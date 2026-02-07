//! Diagnostic and check commands.
//!
//! Status overview and git history auditing.

mod audit;
mod status;

// Re-export command functions
pub use audit::execute as audit;
pub use status::execute as status;
