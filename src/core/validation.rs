//! Input validation for burrow operations.
//!
//! Validates secret keys, values, and other user inputs.

use crate::error::{Result, ValidationError};

/// Validate a secret key name.
///
/// Secret keys must be valid environment variable names:
/// - Only A-Z, 0-9, and underscore
/// - Cannot start with a digit
/// - Cannot be empty
///
/// # Arguments
///
/// * `key` - The key name to validate
///
/// # Errors
///
/// Returns `ValidationError` if the key is invalid.
pub fn validate_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(ValidationError::EmptyKey.into());
    }

    // Check first character - must not be a digit
    if let Some(first_char) = key.chars().next() {
        if first_char.is_ascii_digit() {
            return Err(ValidationError::InvalidKey {
                key: key.to_string(),
                reason: "cannot start with a digit".to_string(),
            }
            .into());
        }
    }

    // Check all characters - must be A-Z, 0-9, or underscore
    for (i, ch) in key.chars().enumerate() {
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            return Err(ValidationError::InvalidKey {
                key: key.to_string(),
                reason: format!(
                    "invalid character '{}' at position {}. Only A-Z, 0-9, and underscore are allowed",
                    ch, i + 1
                ),
            }
            .into());
        }
    }

    Ok(())
}

/// Validate a secret value.
///
/// Secret values cannot be empty.
///
/// # Arguments
///
/// * `key` - The key name (for error messages)
/// * `value` - The value to validate
///
/// # Errors
///
/// Returns `ValidationError` if the value is empty.
pub fn validate_value(key: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(ValidationError::EmptyValue(key.to_string()).into());
    }

    Ok(())
}

/// Validate file permissions (Unix only).
///
/// Checks that a file has the expected permissions mode.
///
/// # Arguments
///
/// * `path` - Path to the file
/// * `expected_mode` - Expected permissions mode (e.g., 0o600)
///
/// # Errors
///
/// Returns `ValidationError` if permissions don't match.
#[cfg(unix)]
pub fn validate_file_permissions(path: &std::path::Path, expected_mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path)?;
    let actual_mode = metadata.permissions().mode() & 0o777;

    if actual_mode != expected_mode {
        return Err(ValidationError::InvalidPermissions {
            path: path.display().to_string(),
            expected: format!("{:o}", expected_mode),
            actual: format!("{:o}", actual_mode),
        }
        .into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_keys() {
        assert!(validate_key("DATABASE_URL").is_ok());
        assert!(validate_key("API_KEY").is_ok());
        assert!(validate_key("SECRET_123").is_ok());
        assert!(validate_key("_PRIVATE").is_ok());
        assert!(validate_key("A").is_ok());
    }

    #[test]
    fn test_invalid_keys() {
        // Empty key
        assert!(validate_key("").is_err());

        // Starting with digit
        assert!(validate_key("123_KEY").is_err());

        // Invalid characters
        assert!(validate_key("API-KEY").is_err());
        assert!(validate_key("API.KEY").is_err());
        assert!(validate_key("API KEY").is_err());
        assert!(validate_key("API@KEY").is_err());
    }

    #[test]
    fn test_valid_values() {
        assert!(validate_value("KEY", "value").is_ok());
        assert!(validate_value("KEY", "with spaces").is_ok());
        assert!(validate_value("KEY", "123").is_ok());
    }

    #[test]
    fn test_invalid_values() {
        assert!(validate_value("KEY", "").is_err());
    }
}
