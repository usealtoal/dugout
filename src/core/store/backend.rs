//! Backend selection logic for identity storage
//!
//! This module determines which storage backend to use (Keychain vs Filesystem)
//! based on platform and explicit user configuration.

use super::{Filesystem, Store};
use tracing::info;
#[cfg(target_os = "macos")]
use tracing::warn;

#[cfg(target_os = "macos")]
use super::keychain::Keychain;

/// Default backend selection
///
/// On macOS: Use Keychain (default) or Filesystem (if DUGOUT_NO_KEYCHAIN=1)
/// Other platforms: Always use Filesystem
///
/// Users can explicitly disable Keychain with DUGOUT_NO_KEYCHAIN=1 environment variable.
pub fn default_backend() -> Box<dyn Store> {
    #[cfg(target_os = "macos")]
    {
        if should_use_keychain() {
            info!("Using macOS Keychain backend (hardware-backed security)");
            if let Ok(keychain) = Keychain::new() {
                return Box::new(keychain);
            } else {
                warn!("Failed to initialize Keychain backend");
            }
        } else {
            info!("Using Filesystem backend (DUGOUT_NO_KEYCHAIN=1)");
        }
    }
    info!("Using Filesystem backend");

    Box::new(Filesystem)
}

/// Determine if Keychain should be used
///
/// macOS: enabled by default, disabled if DUGOUT_NO_KEYCHAIN is set
/// Other platforms: always disabled (no keychain support)
#[cfg(target_os = "macos")]
fn should_use_keychain() -> bool {
    // Use keychain unless explicitly disabled
    std::env::var("DUGOUT_NO_KEYCHAIN").is_err()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_backend_returns_store() {
        let backend = default_backend();
        // Just verify we can create a backend and call methods on it
        assert!(!backend.has_key("nonexistent")); // Should return false for non-existent key
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_should_use_keychain_by_default() {
        // Remove env var if it exists
        std::env::remove_var("DUGOUT_NO_KEYCHAIN");
        assert!(should_use_keychain());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_should_use_keychain_disabled_by_env() {
        std::env::set_var("DUGOUT_NO_KEYCHAIN", "1");
        assert!(!should_use_keychain());
        std::env::remove_var("DUGOUT_NO_KEYCHAIN");
    }
}
