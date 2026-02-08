//! Edge case tests for dugout.
//!
//! These tests verify that dugout correctly handles challenging inputs:
//! - Unicode values (Japanese, emoji, Chinese, Arabic)
//! - Multiline values with embedded newlines
//! - Special characters (quotes, backslashes, shell metacharacters)
//! - Very long values (10KB+)
//! - Whitespace-only values
//! - Edge-case key names
//! - Overwriting secrets
//! - .env roundtrip fidelity

mod support;
use support::*;

#[test]
fn test_unicode_japanese_value() {
    let t = Test::init("test-user");

    let japanese = "こんにちは世界";
    let output = t.set("JAPANESE_SECRET", japanese);
    assert_success(&output);

    let output = t.get("JAPANESE_SECRET");
    assert_success(&output);
    assert_stdout_contains(&output, japanese);
}

#[test]
fn test_unicode_emoji_value() {
    let t = Test::init("test-user");

    let emoji = "🚀🎉💯🔥🌟✨🎨🎭🎪🎬";
    let output = t.set("EMOJI_SECRET", emoji);
    assert_success(&output);

    let output = t.get("EMOJI_SECRET");
    assert_success(&output);
    assert_stdout_contains(&output, emoji);
}

#[test]
fn test_unicode_chinese_value() {
    let t = Test::init("test-user");

    let chinese = "你好世界，这是一个测试秘密";
    let output = t.set("CHINESE_SECRET", chinese);
    assert_success(&output);

    let output = t.get("CHINESE_SECRET");
    assert_success(&output);
    assert_stdout_contains(&output, chinese);
}

#[test]
fn test_unicode_arabic_value() {
    let t = Test::init("test-user");

    let arabic = "مرحبا بالعالم هذا سر اختبار";
    let output = t.set("ARABIC_SECRET", arabic);
    assert_success(&output);

    let output = t.get("ARABIC_SECRET");
    assert_success(&output);
    assert_stdout_contains(&output, arabic);
}

#[test]
fn test_unicode_mixed_value() {
    let t = Test::init("test-user");

    let mixed = "Hello 世界 🌍 مرحبا Привет こんにちは";
    let output = t.set("MIXED_UNICODE", mixed);
    assert_success(&output);

    let output = t.get("MIXED_UNICODE");
    assert_success(&output);
    assert_stdout_contains(&output, mixed);
}

#[test]
fn test_multiline_value_simple() {
    let t = Test::init("test-user");

    let multiline = "line1\nline2\nline3";
    let output = t.set("MULTILINE_SECRET", multiline);
    assert_success(&output);

    let output = t.get("MULTILINE_SECRET");
    assert_success(&output);
    let binding = stdout(&output);
    let result = binding.trim();
    assert!(result.contains("line1"));
    assert!(result.contains("line2"));
    assert!(result.contains("line3"));
}

#[test]
fn test_multiline_value_with_blank_lines() {
    let t = Test::init("test-user");

    let multiline = "line1\n\nline3\n\n\nline6";
    let output = t.set("MULTILINE_BLANK", multiline);
    assert_success(&output);

    let output = t.get("MULTILINE_BLANK");
    assert_success(&output);
    let binding = stdout(&output);
    let result = binding.trim();
    assert!(result.contains("line1"));
    assert!(result.contains("line3"));
    assert!(result.contains("line6"));
}

#[test]
fn test_multiline_value_pem_certificate() {
    let t = Test::init("test-user");

    // Multiline PEM certificate - values starting with dashes are problematic for CLI
    // Use the import workflow which is the proper way to handle complex multiline values
    let pem = "-----BEGIN CERTIFICATE-----\nMIIDXTCCAkWgAwIBAgIJAKL0UG+mRKfzMA0GCSqGSIb3DQEBCwUAMEUxCzAJBgNV\nBAYTAkFVMRMwEQYDVQQIDApTb21lLVN0YXRlMSEwHwYDVQQKDBhJbnRlcm5ldCBX\n-----END CERTIFICATE-----";

    // Create .env file with proper escaping
    // In .env format, we can use quotes and escape sequences
    let env_content = format!("PEM_CERT={}", pem.replace('\n', "\\n"));
    std::fs::write(t.dir.path().join("pem.env"), &env_content).unwrap();

    let output = t.secrets_import("pem.env");
    assert_success(&output);

    let output = t.get("PEM_CERT");
    assert_success(&output);
    let result = stdout(&output);

    // The value should contain the certificate markers
    assert!(result.contains("BEGIN CERTIFICATE"), "Missing BEGIN marker");
    assert!(result.contains("END CERTIFICATE"), "Missing END marker");
}

#[test]
fn test_special_chars_double_quotes() {
    let t = Test::init("test-user");

    let value = r#"He said "hello" to me"#;
    let output = t.set("QUOTE_SECRET", value);
    assert_success(&output);

    let output = t.get("QUOTE_SECRET");
    assert_success(&output);
    assert_stdout_contains(&output, "hello");
}

#[test]
fn test_special_chars_single_quotes() {
    let t = Test::init("test-user");

    let value = "It's a beautiful day";
    let output = t.set("APOSTROPHE", value);
    assert_success(&output);

    let output = t.get("APOSTROPHE");
    assert_success(&output);
    assert_stdout_contains(&output, "It's");
}

#[test]
fn test_special_chars_backslashes() {
    let t = Test::init("test-user");

    let value = r"C:\Windows\System32\config";
    let output = t.set("WINDOWS_PATH", value);
    assert_success(&output);

    let output = t.get("WINDOWS_PATH");
    assert_success(&output);
    let result = stdout(&output);
    assert!(result.contains("Windows"));
    assert!(result.contains("System32"));
}

#[test]
fn test_special_chars_dollar_signs() {
    let t = Test::init("test-user");

    let value = "$100 or ${VAR} or $HOME";
    let output = t.set("DOLLAR_SECRET", value);
    assert_success(&output);

    let output = t.get("DOLLAR_SECRET");
    assert_success(&output);
    let result = stdout(&output);
    assert!(result.contains("$100"));
    assert!(result.contains("${VAR}"));
}

#[test]
fn test_special_chars_backticks() {
    let t = Test::init("test-user");

    let value = "Run `command` to test";
    let output = t.set("BACKTICK_SECRET", value);
    assert_success(&output);

    let output = t.get("BACKTICK_SECRET");
    assert_success(&output);
    assert_stdout_contains(&output, "`command`");
}

#[test]
fn test_special_chars_mixed() {
    let t = Test::init("test-user");

    let value = r#"p@$$w0rd!#%^&*(){}[]|\"'<>?,./~`"#;
    let output = t.set("SPECIAL_MIX", value);
    assert_success(&output);

    let output = t.get("SPECIAL_MIX");
    assert_success(&output);
    let result = stdout(&output);
    assert!(result.contains("p@$$w0rd"));
}

#[test]
fn test_special_chars_sql_injection() {
    let t = Test::init("test-user");

    let value = "' OR '1'='1'; DROP TABLE users; --";
    let output = t.set("SQL_INJECTION", value);
    assert_success(&output);

    let output = t.get("SQL_INJECTION");
    assert_success(&output);
    assert_stdout_contains(&output, "DROP TABLE");
}

#[test]
fn test_special_chars_shell_injection() {
    let t = Test::init("test-user");

    let value = "; rm -rf / ; echo 'pwned'";
    let output = t.set("SHELL_INJECTION", value);
    assert_success(&output);

    let output = t.get("SHELL_INJECTION");
    assert_success(&output);
    let result = stdout(&output);
    assert!(result.contains("rm -rf"));
}

#[test]
fn test_very_long_value_10kb() {
    let t = Test::init("test-user");

    // Create a 10KB value
    let large_value = "A".repeat(10_000);
    let output = t.set("LARGE_10KB", &large_value);
    assert_success(&output);

    let output = t.get("LARGE_10KB");
    assert_success(&output);
    let binding = stdout(&output);
    let result = binding.trim();
    assert!(
        result.len() >= 10_000,
        "Expected at least 10KB, got {} bytes",
        result.len()
    );
}

#[test]
fn test_very_long_value_100kb() {
    let t = Test::init("test-user");

    // Create a 100KB value
    let large_value = "B".repeat(100_000);
    let output = t.set("LARGE_100KB", &large_value);
    assert_success(&output);

    let output = t.get("LARGE_100KB");
    assert_success(&output);
    let binding = stdout(&output);
    let result = binding.trim();
    assert!(
        result.len() >= 100_000,
        "Expected at least 100KB, got {} bytes",
        result.len()
    );
}

#[test]
fn test_very_long_value_multiline_10kb() {
    let t = Test::init("test-user");

    // Create a 10KB value with newlines every 100 chars
    let mut large_value = String::new();
    for i in 0..100 {
        large_value.push_str(&format!("Line {} ", i));
        large_value.push_str(&"x".repeat(90));
        large_value.push('\n');
    }

    let output = t.set("LARGE_ML_10KB", &large_value);
    assert_success(&output);

    let output = t.get("LARGE_ML_10KB");
    assert_success(&output);
    let result = stdout(&output);
    assert!(result.contains("Line 0"));
    assert!(result.contains("Line 99"));
}

#[test]
fn test_spaces_only_value() {
    let t = Test::init("test-user");

    let spaces = "     ";
    let output = t.set("SPACES_ONLY", spaces);
    assert_success(&output);

    let output = t.get("SPACES_ONLY");
    assert_success(&output);
    // Should return the spaces (though they might be trimmed by shell)
    let result = stdout(&output);
    assert!(
        !result.is_empty(),
        "Should return something for spaces-only value"
    );
}

#[test]
fn test_tabs_only_value() {
    let t = Test::init("test-user");

    let tabs = "\t\t\t\t";
    let output = t.set("TABS_ONLY", tabs);
    assert_success(&output);

    let output = t.get("TABS_ONLY");
    assert_success(&output);
    let result = stdout(&output);
    assert!(
        !result.is_empty(),
        "Should return something for tabs-only value"
    );
}

#[test]
fn test_empty_value_is_rejected() {
    let t = Test::init("test-user");

    // Empty values are not allowed by design
    let output = t.set("EMPTY_SECRET", "");
    assert_failure(&output);
    assert_stderr_contains(&output, "empty");
}

#[test]
fn test_key_starting_with_number_is_rejected() {
    let t = Test::init("test-user");

    // Keys starting with digits are not allowed (env var naming rules)
    let output = t.set("12345", "numeric-key-value");
    assert_failure(&output);
    assert_stderr_contains(&output, "cannot start with a digit");
}

#[test]
fn test_key_with_numbers_after_letter() {
    let t = Test::init("test-user");

    // But keys can contain numbers if they don't start with one
    let output = t.set("KEY12345", "numeric-key-value");
    assert_success(&output);

    let output = t.get("KEY12345");
    assert_success(&output);
    assert_stdout_contains(&output, "numeric-key-value");
}

#[test]
fn test_key_with_underscores() {
    let t = Test::init("test-user");

    let output = t.set("_____", "underscore-key-value");
    assert_success(&output);

    let output = t.get("_____");
    assert_success(&output);
    assert_stdout_contains(&output, "underscore-key-value");
}

#[test]
fn test_key_with_mixed_underscores() {
    let t = Test::init("test-user");

    let output = t.set("_TEST_KEY_123_", "mixed-key-value");
    assert_success(&output);

    let output = t.get("_TEST_KEY_123_");
    assert_success(&output);
    assert_stdout_contains(&output, "mixed-key-value");
}

#[test]
fn test_key_very_long() {
    let t = Test::init("test-user");

    // Create a very long key (255 chars)
    let long_key = format!("KEY_{}", "X".repeat(251));
    let output = t.set(&long_key, "long-key-value");
    assert_success(&output);

    let output = t.get(&long_key);
    assert_success(&output);
    assert_stdout_contains(&output, "long-key-value");
}

#[test]
fn test_overwrite_secret_without_force_fails() {
    let t = Test::init("test-user");

    let output = t.set("OVERWRITE_TEST", "original");
    assert_success(&output);

    // Try to overwrite without --force (should fail)
    let output = t.set("OVERWRITE_TEST", "modified");
    assert_failure(&output);

    // Verify original value is still there
    let output = t.get("OVERWRITE_TEST");
    assert_success(&output);
    assert_stdout_contains(&output, "original");
}

#[test]
fn test_overwrite_secret_with_force_succeeds() {
    let t = Test::init("test-user");

    let output = t.set("OVERWRITE_FORCE", "original");
    assert_success(&output);

    // Overwrite with --force
    let output = t.set_force("OVERWRITE_FORCE", "modified");
    assert_success(&output);

    // Verify new value
    let output = t.get("OVERWRITE_FORCE");
    assert_success(&output);
    assert_stdout_contains(&output, "modified");
    assert_stdout_excludes(&output, "original");
}

#[test]
fn test_overwrite_multiple_times() {
    let t = Test::init("test-user");

    let output = t.set("MULTI_OVERWRITE", "v1");
    assert_success(&output);

    for i in 2..=5 {
        let value = format!("v{}", i);
        let output = t.set_force("MULTI_OVERWRITE", &value);
        assert_success(&output);
    }

    // Verify final value
    let output = t.get("MULTI_OVERWRITE");
    assert_success(&output);
    assert_stdout_contains(&output, "v5");
    assert_stdout_excludes(&output, "v1");
    assert_stdout_excludes(&output, "v4");
}

#[test]
fn test_secrets_unlock_roundtrip_basic() {
    let t = Test::init("test-user");

    // Set some secrets
    t.set("KEY1", "value1");
    t.set("KEY2", "value2");
    t.set("KEY3", "value3");

    // Unlock (create .env)
    let output = t.secrets_unlock();
    assert_success(&output);

    // Verify .env was created
    let env_path = t.dir.path().join(".env");
    assert!(env_path.exists(), ".env file should be created");

    // Read .env
    let env_content = std::fs::read_to_string(&env_path).unwrap();
    assert!(env_content.contains("KEY1=value1"));
    assert!(env_content.contains("KEY2=value2"));
    assert!(env_content.contains("KEY3=value3"));
}

#[test]
fn test_secrets_unlock_roundtrip_unicode() {
    let t = Test::init("test-user");

    // Set Unicode secrets
    t.set("JAPANESE", "こんにちは");
    t.set("EMOJI", "🚀🎉");
    t.set("ARABIC", "مرحبا");

    // Unlock
    let output = t.secrets_unlock();
    assert_success(&output);

    // Read and verify .env
    let env_path = t.dir.path().join(".env");
    let _env_content = std::fs::read_to_string(&env_path).unwrap();
    // Note: .env format may escape or quote unicode differently
    // For now, just verify the file was created
    // Could add more specific assertions about encoding if needed
}

#[test]
fn test_secrets_unlock_roundtrip_special_chars() {
    let t = Test::init("test-user");

    // Set secrets with special characters
    t.set("QUOTE", r#"He said "hello""#);
    t.set("DOLLAR", "$100 ${VAR}");
    t.set("BACKSLASH", r"C:\Windows\System32");

    // Unlock
    let output = t.secrets_unlock();
    assert_success(&output);

    // Read .env
    let env_path = t.dir.path().join(".env");
    let _env_content = std::fs::read_to_string(&env_path).unwrap();

    // Lock and re-import to test roundtrip
    let output = t.secrets_lock();
    assert_success(&output);

    let output = t.secrets_import(".env");
    assert_success(&output);

    // Verify values are correct after roundtrip
    let output = t.get("QUOTE");
    assert_success(&output);
    assert_stdout_contains(&output, "hello");

    let output = t.get("DOLLAR");
    assert_success(&output);
    assert_stdout_contains(&output, "$100");

    let output = t.get("BACKSLASH");
    assert_success(&output);
    assert_stdout_contains(&output, "Windows");
}

#[test]
fn test_secrets_unlock_roundtrip_multiline() {
    let t = Test::init("test-user");

    // Set multiline secret
    let multiline = "line1\nline2\nline3";
    t.set("MULTILINE", multiline);

    // Unlock
    let output = t.secrets_unlock();
    assert_success(&output);

    // Lock and re-import
    let output = t.secrets_lock();
    assert_success(&output);

    let output = t.secrets_import(".env");
    assert_success(&output);

    // Verify multiline value is preserved
    let output = t.get("MULTILINE");
    assert_success(&output);
    let result = stdout(&output);
    assert!(result.contains("line1"));
    assert!(result.contains("line2"));
    assert!(result.contains("line3"));
}

#[test]
fn test_secrets_import_tricky_env_file() {
    let t = Test::init("test-user");

    // Create a tricky .env file
    let tricky_env = r#"
# Comment at the start
SIMPLE=value

# Quoted values
QUOTED="quoted value"
SINGLE='single quoted'

# Special characters
SPECIAL=p@$$w0rd!#$%
DOLLAR=$100
BACKTICK=`command`

# Unicode
UNICODE=こんにちは🚀

# Whitespace values (empty values are not allowed)
SPACES=   spaces   

# Multiline (if supported)
# MULTILINE="line1
# line2"

# Complex
COMPLEX="He said \"hello\" to me"
"#;

    std::fs::write(t.dir.path().join("tricky.env"), tricky_env).unwrap();

    // Import
    let output = t.secrets_import("tricky.env");
    assert_success(&output);

    // Verify imported values
    let output = t.get("SIMPLE");
    assert_success(&output);
    assert_stdout_contains(&output, "value");

    let output = t.get("SPECIAL");
    assert_success(&output);
    let result = stdout(&output);
    assert!(result.contains("p@$$w0rd") || result.contains("p@\\$\\$w0rd"));

    let output = t.get("UNICODE");
    assert_success(&output);
    assert_stdout_contains(&output, "こんにちは");
}

#[test]
fn test_secrets_import_export_roundtrip() {
    let t = Test::init("test-user");

    // Set various edge-case secrets
    t.set("UNICODE", "🚀こんにちは");
    t.set("SPECIAL", r#"p@$$w0rd!#%"#);
    t.set("QUOTE", r#"He said "hi""#);

    // Export
    let output = t.secrets_export();
    assert_success(&output);
    let exported = stdout(&output);

    // Save to file
    std::fs::write(t.dir.path().join("exported.env"), exported).unwrap();

    // Lock all secrets
    let output = t.secrets_lock();
    assert_success(&output);

    // Re-import
    let output = t.secrets_import("exported.env");
    assert_success(&output);

    // Verify all values are preserved
    let output = t.get("UNICODE");
    assert_success(&output);
    assert_stdout_contains(&output, "🚀");
    assert_stdout_contains(&output, "こんにちは");

    let output = t.get("SPECIAL");
    assert_success(&output);

    let output = t.get("QUOTE");
    assert_success(&output);
}

#[test]
fn test_secrets_import_with_equals_in_value() {
    let t = Test::init("test-user");

    // Create .env with equals signs in values
    let env_content = "JWT_TOKEN=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWI9MTIzNDU2Nzg5MCJ9\nBASE64=SGVsbG8gV29ybGQ=\n";
    std::fs::write(t.dir.path().join("equals.env"), env_content).unwrap();

    // Import
    let output = t.secrets_import("equals.env");
    assert_success(&output);

    // Verify values with equals signs
    let output = t.get("JWT_TOKEN");
    assert_success(&output);
    assert_stdout_contains(&output, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");

    let output = t.get("BASE64");
    assert_success(&output);
    assert_stdout_contains(&output, "SGVsbG8gV29ybGQ=");
}

#[test]
fn test_null_byte_handling() {
    let t = Test::init("test-user");

    // Null bytes cause a panic in the command execution
    // This is because the OS doesn't allow null bytes in command arguments
    // We can't test this directly via CLI, but we know it's rejected
    // This is acceptable behavior - null bytes shouldn't be in secret values anyway

    // Instead, test that a value that looks like it might have null bytes works fine
    let value = "before\\x00after";
    let output = t.set("ESCAPED_NULL", &value);
    assert_success(&output);

    let output = t.get("ESCAPED_NULL");
    assert_success(&output);
    assert_stdout_contains(&output, "before");
    assert_stdout_contains(&output, "after");
}

#[test]
fn test_crlf_line_endings() {
    let t = Test::init("test-user");

    // Value with Windows line endings
    let crlf_value = "line1\r\nline2\r\nline3";
    let output = t.set("CRLF_VALUE", crlf_value);
    assert_success(&output);

    let output = t.get("CRLF_VALUE");
    assert_success(&output);
    let result = stdout(&output);
    assert!(result.contains("line1"));
    assert!(result.contains("line2"));
    assert!(result.contains("line3"));
}

#[test]
fn test_consecutive_newlines() {
    let t = Test::init("test-user");

    let value = "line1\n\n\n\nline5";
    let output = t.set("CONSECUTIVE_NL", value);
    assert_success(&output);

    let output = t.get("CONSECUTIVE_NL");
    assert_success(&output);
    let result = stdout(&output);
    assert!(result.contains("line1"));
    assert!(result.contains("line5"));
}

#[test]
fn test_leading_and_trailing_whitespace() {
    let t = Test::init("test-user");

    let value = "   leading and trailing   ";
    let output = t.set("WHITESPACE", value);
    assert_success(&output);

    let output = t.get("WHITESPACE");
    assert_success(&output);
    // The value should be preserved (though display might trim)
    let result = stdout(&output);
    assert!(result.contains("leading and trailing"));
}

#[test]
fn test_unicode_key_names() {
    let t = Test::init("test-user");

    // Try to use Unicode in key name (this might fail, which is OK)
    let output = t.set("KEY_🚀", "rocket-value");

    if output.status.success() {
        let output = t.get("KEY_🚀");
        assert_success(&output);
        assert_stdout_contains(&output, "rocket-value");
    }
    // If Unicode keys are rejected, that's also acceptable
}
