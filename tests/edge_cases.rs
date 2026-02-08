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
    assert_roundtrip(&t, "JAPANESE_SECRET", "こんにちは世界");
}

#[test]
fn test_unicode_emoji_value() {
    let t = Test::init("test-user");
    assert_roundtrip(&t, "EMOJI_SECRET", "🚀🎉💯🔥🌟✨🎨🎭🎪🎬");
}

#[test]
fn test_unicode_chinese_value() {
    let t = Test::init("test-user");
    assert_roundtrip(&t, "CHINESE_SECRET", "你好世界，这是一个测试秘密");
}

#[test]
fn test_unicode_arabic_value() {
    let t = Test::init("test-user");
    assert_roundtrip(&t, "ARABIC_SECRET", "مرحبا بالعالم هذا سر اختبار");
}

#[test]
fn test_unicode_mixed_value() {
    let t = Test::init("test-user");
    assert_roundtrip(&t, "MIXED_UNICODE", "Hello 世界 🌍 مرحبا Привет こんにちは");
}

#[test]
fn test_multiline_value_simple() {
    let t = Test::init("test-user");

    let multiline = "line1\nline2\nline3";
    let output = t.set("MULTILINE_SECRET", multiline);
    assert_success(&output);

    let output = t.get("MULTILINE_SECRET");
    assert_success(&output);
    assert_stdout_contains(&output, "line1");
    assert_stdout_contains(&output, "line2");
    assert_stdout_contains(&output, "line3");
}

#[test]
fn test_multiline_value_with_blank_lines() {
    let t = Test::init("test-user");

    let multiline = "line1\n\nline3\n\n\nline6";
    let output = t.set("MULTILINE_BLANK", multiline);
    assert_success(&output);

    let output = t.get("MULTILINE_BLANK");
    assert_success(&output);
    assert_stdout_contains(&output, "line1");
    assert_stdout_contains(&output, "line3");
    assert_stdout_contains(&output, "line6");
}

#[test]
fn test_multiline_value_pem_certificate() {
    let t = Test::init("test-user");

    let pem = "-----BEGIN CERTIFICATE-----\nMIIDXTCCAkWgAwIBAgIJAKL0UG+mRKfzMA0GCSqGSIb3DQEBCwUAMEUxCzAJBgNV\nBAYTAkFVMRMwEQYDVQQIDApTb21lLVN0YXRlMSEwHwYDVQQKDBhJbnRlcm5ldCBX\n-----END CERTIFICATE-----";

    let env_content = format!("PEM_CERT={}", pem.replace('\n', "\\n"));
    std::fs::write(t.dir.path().join("pem.env"), &env_content).unwrap();

    let output = t.secrets_import("pem.env");
    assert_success(&output);

    let output = t.get("PEM_CERT");
    assert_success(&output);
    let result = stdout(&output);

    assert!(result.contains("BEGIN CERTIFICATE"), "Missing BEGIN marker");
    assert!(result.contains("END CERTIFICATE"), "Missing END marker");
}

#[test]
fn test_special_chars_double_quotes() {
    let t = Test::init("test-user");
    assert_roundtrip(&t, "QUOTE_SECRET", r#"He said "hello" to me"#);
}

#[test]
fn test_special_chars_single_quotes() {
    let t = Test::init("test-user");
    assert_roundtrip(&t, "APOSTROPHE", "It's a beautiful day");
}

#[test]
fn test_special_chars_backslashes() {
    let t = Test::init("test-user");

    let value = r"C:\Windows\System32\config";
    let output = t.set("WINDOWS_PATH", value);
    assert_success(&output);

    let output = t.get("WINDOWS_PATH");
    assert_success(&output);
    assert_stdout_contains(&output, "Windows");
    assert_stdout_contains(&output, "System32");
}

#[test]
fn test_special_chars_dollar_signs() {
    let t = Test::init("test-user");

    let value = "$100 or ${VAR} or $HOME";
    let output = t.set("DOLLAR_SECRET", value);
    assert_success(&output);

    let output = t.get("DOLLAR_SECRET");
    assert_success(&output);
    assert_stdout_contains(&output, "$100");
    assert_stdout_contains(&output, "${VAR}");
}

#[test]
fn test_special_chars_backticks() {
    let t = Test::init("test-user");
    assert_roundtrip(&t, "BACKTICK_SECRET", "Run `command` to test");
}

#[test]
fn test_special_chars_mixed() {
    let t = Test::init("test-user");

    let value = r#"p@$$w0rd!#%^&*(){}[]|\"'<>?,./~`"#;
    let output = t.set("SPECIAL_MIX", value);
    assert_success(&output);

    let output = t.get("SPECIAL_MIX");
    assert_success(&output);
    assert_stdout_contains(&output, "p@$$w0rd");
}

#[test]
fn test_special_chars_sql_injection() {
    let t = Test::init("test-user");
    assert_roundtrip(&t, "SQL_INJECTION", "' OR '1'='1'; DROP TABLE users; --");
}

#[test]
fn test_special_chars_shell_injection() {
    let t = Test::init("test-user");

    let value = "; rm -rf / ; echo 'pwned'";
    let output = t.set("SHELL_INJECTION", value);
    assert_success(&output);

    let output = t.get("SHELL_INJECTION");
    assert_success(&output);
    assert_stdout_contains(&output, "rm -rf");
}

#[test]
fn test_very_long_value_10kb() {
    let t = Test::init("test-user");

    let large_value = "A".repeat(10_000);
    let output = t.set("LARGE_10KB", &large_value);
    assert_success(&output);

    let output = t.get("LARGE_10KB");
    assert_success(&output);
    let result = stdout(&output).trim().to_string();
    assert!(
        result.len() >= 10_000,
        "Expected at least 10KB, got {} bytes",
        result.len()
    );
}

#[test]
#[cfg_attr(windows, ignore)]
fn test_very_long_value_100kb() {
    let t = Test::init("test-user");

    let large_value = "B".repeat(100_000);
    let output = t.set("LARGE_100KB", &large_value);
    assert_success(&output);

    let output = t.get("LARGE_100KB");
    assert_success(&output);
    let result = stdout(&output).trim().to_string();
    assert!(
        result.len() >= 100_000,
        "Expected at least 100KB, got {} bytes",
        result.len()
    );
}

#[test]
fn test_very_long_value_multiline_10kb() {
    let t = Test::init("test-user");

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
    assert_stdout_contains(&output, "Line 0");
    assert_stdout_contains(&output, "Line 99");
}

#[test]
fn test_spaces_only_value() {
    let t = Test::init("test-user");

    let spaces = "     ";
    let output = t.set("SPACES_ONLY", spaces);
    assert_success(&output);

    let output = t.get("SPACES_ONLY");
    assert_success(&output);
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

    let output = t.set("EMPTY_SECRET", "");
    assert_failure(&output);
    assert_stderr_contains(&output, "empty");
}

#[test]
fn test_key_starting_with_number_is_rejected() {
    let t = Test::init("test-user");

    let output = t.set("12345", "numeric-key-value");
    assert_failure(&output);
    assert_stderr_contains(&output, "cannot start with a digit");
}

#[test]
fn test_key_with_numbers_after_letter() {
    let t = Test::init("test-user");
    assert_roundtrip(&t, "KEY12345", "numeric-key-value");
}

#[test]
fn test_key_with_underscores() {
    let t = Test::init("test-user");
    assert_roundtrip(&t, "_____", "underscore-key-value");
}

#[test]
fn test_key_with_mixed_underscores() {
    let t = Test::init("test-user");
    assert_roundtrip(&t, "_TEST_KEY_123_", "mixed-key-value");
}

#[test]
fn test_key_very_long() {
    let t = Test::init("test-user");

    let long_key = format!("KEY_{}", "X".repeat(251));
    assert_roundtrip(&t, &long_key, "long-key-value");
}

#[test]
fn test_overwrite_secret_without_force_fails() {
    let t = Test::init("test-user");

    let output = t.set("OVERWRITE_TEST", "original");
    assert_success(&output);

    let output = t.set("OVERWRITE_TEST", "modified");
    assert_failure(&output);

    let output = t.get("OVERWRITE_TEST");
    assert_success(&output);
    assert_stdout_contains(&output, "original");
}

#[test]
fn test_overwrite_secret_with_force_succeeds() {
    let t = Test::init("test-user");

    let output = t.set("OVERWRITE_FORCE", "original");
    assert_success(&output);

    let output = t.set_force("OVERWRITE_FORCE", "modified");
    assert_success(&output);

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

    let output = t.get("MULTI_OVERWRITE");
    assert_success(&output);
    assert_stdout_contains(&output, "v5");
    assert_stdout_excludes(&output, "v1");
    assert_stdout_excludes(&output, "v4");
}

#[test]
fn test_secrets_unlock_roundtrip_basic() {
    let t = Test::init("test-user");

    t.set("KEY1", "value1");
    t.set("KEY2", "value2");
    t.set("KEY3", "value3");

    let output = t.secrets_unlock();
    assert_success(&output);

    let env_path = t.dir.path().join(".env");
    assert!(env_path.exists(), ".env file should be created");

    let env_content = std::fs::read_to_string(&env_path).unwrap();
    assert!(env_content.contains("KEY1=value1"));
    assert!(env_content.contains("KEY2=value2"));
    assert!(env_content.contains("KEY3=value3"));
}

#[test]
fn test_secrets_unlock_roundtrip_unicode() {
    let t = Test::init("test-user");

    let test_cases = [
        ("JAPANESE", "こんにちは"),
        ("EMOJI", "🚀🎉"),
        ("ARABIC", "مرحبا"),
    ];

    for (key, value) in &test_cases {
        t.set(key, value);
    }

    let output = t.secrets_unlock();
    assert_success(&output);

    let env_path = t.dir.path().join(".env");
    let env_content = std::fs::read_to_string(&env_path).unwrap();

    let output = t.secrets_lock();
    assert_success(&output);

    let output = t.secrets_import(".env");
    assert_success(&output);

    for (key, value) in &test_cases {
        let output = t.get(key);
        assert_success(&output);
        assert_stdout_contains(&output, value);
    }
}

#[test]
fn test_secrets_unlock_roundtrip_special_chars() {
    let t = Test::init("test-user");

    t.set("QUOTE", r#"He said "hello""#);
    t.set("DOLLAR", "$100 ${VAR}");
    t.set("BACKSLASH", r"C:\Windows\System32");

    let output = t.secrets_unlock();
    assert_success(&output);

    let env_path = t.dir.path().join(".env");
    let _env_content = std::fs::read_to_string(&env_path).unwrap();

    let output = t.secrets_lock();
    assert_success(&output);

    let output = t.secrets_import(".env");
    assert_success(&output);

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

    let multiline = "line1\nline2\nline3";
    t.set("MULTILINE", multiline);

    let output = t.secrets_unlock();
    assert_success(&output);

    let output = t.secrets_lock();
    assert_success(&output);

    let output = t.secrets_import(".env");
    assert_success(&output);

    let output = t.get("MULTILINE");
    assert_success(&output);
    assert_stdout_contains(&output, "line1");
    assert_stdout_contains(&output, "line2");
    assert_stdout_contains(&output, "line3");
}

#[test]
fn test_secrets_import_tricky_env_file() {
    let t = Test::init("test-user");

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

    let output = t.secrets_import("tricky.env");
    assert_success(&output);

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

    t.set("UNICODE", "🚀こんにちは");
    t.set("SPECIAL", r#"p@$$w0rd!#%"#);
    t.set("QUOTE", r#"He said "hi""#);

    let output = t.secrets_export();
    assert_success(&output);
    let exported = stdout(&output);

    std::fs::write(t.dir.path().join("exported.env"), exported).unwrap();

    let output = t.secrets_lock();
    assert_success(&output);

    let output = t.secrets_import("exported.env");
    assert_success(&output);

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

    let env_content = "JWT_TOKEN=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWI9MTIzNDU2Nzg5MCJ9\nBASE64=SGVsbG8gV29ybGQ=\n";
    std::fs::write(t.dir.path().join("equals.env"), env_content).unwrap();

    let output = t.secrets_import("equals.env");
    assert_success(&output);

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

    let crlf_value = "line1\r\nline2\r\nline3";
    let output = t.set("CRLF_VALUE", crlf_value);
    assert_success(&output);

    let output = t.get("CRLF_VALUE");
    assert_success(&output);
    assert_stdout_contains(&output, "line1");
    assert_stdout_contains(&output, "line2");
    assert_stdout_contains(&output, "line3");
}

#[test]
fn test_consecutive_newlines() {
    let t = Test::init("test-user");
    assert_roundtrip(&t, "CONSECUTIVE_NL", "line1\n\n\n\nline5");
}

#[test]
fn test_leading_and_trailing_whitespace() {
    let t = Test::init("test-user");

    let value = "   leading and trailing   ";
    let output = t.set("WHITESPACE", value);
    assert_success(&output);

    let output = t.get("WHITESPACE");
    assert_success(&output);
    assert_stdout_contains(&output, "leading and trailing");
}

#[test]
fn test_unicode_key_names() {
    let t = Test::init("test-user");

    let output = t.set("KEY_🚀", "rocket-value");

    if output.status.success() {
        let output = t.get("KEY_🚀");
        assert_success(&output);
        assert_stdout_contains(&output, "rocket-value");
    }
}
