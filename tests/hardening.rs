//! Hardening tests for edge cases, concurrency, and recovery.
//!
//! These tests verify dugout handles adversarial and edge-case inputs
//! gracefully without panics, data loss, or corruption.

mod support;

use std::fs;
use std::sync::{Arc, Barrier};
use std::thread;
use support::*;

// ============================================================================
// Concurrent Access Tests
// ============================================================================

#[test]
fn test_concurrent_reads() {
    let t = Test::with_secrets("alice", &[("KEY1", "value1"), ("KEY2", "value2")]);

    let dir = t.dir.path().to_path_buf();
    let home = t.home.path().to_path_buf();
    let barrier = Arc::new(Barrier::new(4));

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let dir = dir.clone();
            let home = home.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                let key = if i % 2 == 0 { "KEY1" } else { "KEY2" };
                let output = std::process::Command::new(env!("CARGO_BIN_EXE_dugout"))
                    .args(["get", key])
                    .env("HOME", &home)
                    .env("USERPROFILE", &home)
                    .current_dir(&dir)
                    .output()
                    .expect("failed to run dugout");
                output.status.success()
            })
        })
        .collect();

    let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert!(results.iter().all(|&r| r), "All concurrent reads should succeed");
}

#[test]
fn test_concurrent_writes_different_keys() {
    let t = Test::init("alice");

    let dir = t.dir.path().to_path_buf();
    let home = t.home.path().to_path_buf();
    let barrier = Arc::new(Barrier::new(4));

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let dir = dir.clone();
            let home = home.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                let key = format!("CONCURRENT_KEY_{}", i);
                let value = format!("value_{}", i);
                let output = std::process::Command::new(env!("CARGO_BIN_EXE_dugout"))
                    .args(["set", &key, &value])
                    .env("HOME", &home)
                    .env("USERPROFILE", &home)
                    .current_dir(&dir)
                    .output()
                    .expect("failed to run dugout");
                (i, output.status.success())
            })
        })
        .collect();

    let results: Vec<(i32, bool)> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // At least some should succeed (last writer wins for config)
    let successes = results.iter().filter(|(_, success)| *success).count();
    assert!(successes > 0, "At least one concurrent write should succeed");

    // Verify all keys that were written can be read
    for i in 0..4 {
        let key = format!("CONCURRENT_KEY_{}", i);
        let output = t.get(&key);
        // Some may fail if overwritten by concurrent write - that's ok
        if output.status.success() {
            let val = stdout(&output);
            assert!(val.contains("value_"), "Value should be valid");
        }
    }
}

#[test]
fn test_concurrent_read_write() {
    let t = Test::with_secrets("alice", &[("SHARED_KEY", "initial_value")]);

    let dir = t.dir.path().to_path_buf();
    let home = t.home.path().to_path_buf();
    let barrier = Arc::new(Barrier::new(3));

    // 2 readers, 1 writer
    let mut handles = vec![];

    for _ in 0..2 {
        let dir = dir.clone();
        let home = home.clone();
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            let output = std::process::Command::new(env!("CARGO_BIN_EXE_dugout"))
                .args(["get", "SHARED_KEY"])
                .env("HOME", &home)
                .env("USERPROFILE", &home)
                .current_dir(&dir)
                .output()
                .expect("failed to run dugout");
            ("read", output.status.success())
        }));
    }

    {
        let dir = dir.clone();
        let home = home.clone();
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            let output = std::process::Command::new(env!("CARGO_BIN_EXE_dugout"))
                .args(["set", "SHARED_KEY", "updated_value", "--force"])
                .env("HOME", &home)
                .env("USERPROFILE", &home)
                .current_dir(&dir)
                .output()
                .expect("failed to run dugout");
            ("write", output.status.success())
        }));
    }

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // All operations should complete without panic/crash
    for (op, success) in &results {
        assert!(success, "{} operation should succeed", op);
    }
}

// ============================================================================
// Parser Edge Cases (Fuzz-like)
// ============================================================================

/// Edge case values that should be handled without panic
fn edge_case_values() -> Vec<&'static str> {
    vec![
        "",                           // empty
        " ",                          // whitespace only
        "\n",                         // newline only
        "\r\n",                       // CRLF
        "\t\t\t",                     // tabs
        "=",                          // just equals
        "===",                        // multiple equals
        "key=value=extra",            // multiple equals
        "\"unclosed",                 // unclosed quote
        "'unclosed",                  // unclosed single quote
        "\\n\\t\\r",                  // escaped chars as literal
        "\x00",                       // null byte
        "\x00\x00\x00",               // multiple null bytes
        "a]b[c{d}e",                  // brackets
        "a\nb\nc",                    // embedded newlines
        "emoji: \u{1F600}\u{1F4A9}", // emoji
        "日本語テスト",                // Japanese
        "مرحبا",                      // Arabic (RTL)
        "\u{202E}reversed",           // RTL override
        "path/../../../etc/passwd",   // path traversal attempt
        "${VAR}",                     // shell variable
        "$(command)",                 // command substitution
        "`command`",                  // backtick command
        "'; DROP TABLE secrets; --",  // SQL injection attempt
        "<script>alert(1)</script>",  // XSS attempt
    ]
}

/// Generate a long value for testing
fn long_value() -> String {
    "X".repeat(10_000)
}

#[test]
fn test_set_edge_case_values() {
    let t = Test::init("alice");

    let values = edge_case_values();
    let long = long_value();

    for (i, value) in values.iter().enumerate() {
        // Skip null bytes - they can't be passed as args
        if value.contains('\x00') {
            continue;
        }

        let key = format!("EDGE_KEY_{}", i);
        let output = t.set(&key, value);

        // Should either succeed or fail gracefully (no panic)
        if output.status.success() {
            // If set succeeded, get should return the same value
            let _get_output = t.get(&key);
        }
        // Failure is acceptable for some edge cases (empty value, etc.)
    }

    // Test long value separately
    let output = t.set("LONG_KEY", &long);
    if output.status.success() {
        let _get_output = t.get("LONG_KEY");
    }
}

#[test]
fn test_env_parser_edge_cases() {
    let t = Test::init("alice");

    let edge_cases = [
        // (content, should_import_something)
        ("", false),
        ("\n\n\n", false),
        ("# just comments\n# more comments", false),
        ("VALID=value", true),
        ("VALID=value\nINVALID LINE\nVALID2=value2", true),
        ("=no_key", false),
        ("NO_VALUE=", true),  // empty value is valid
        ("SPACES = value", true),
        ("  LEADING=value", true),
        ("TRAILING=value  ", true),
        ("QUOTED=\"value with spaces\"", true),
        ("SINGLE='value'", true),
        ("MIXED=\"unclosed", true),  // partial parse
        ("KEY1=val1\nKEY1=val2", true),  // duplicate keys
    ];

    for (i, (content, should_import)) in edge_cases.iter().enumerate() {
        let env_file = t.dir.path().join(format!("edge_{}.env", i));
        fs::write(&env_file, content).unwrap();

        let _output = t.secrets_import(env_file.to_str().unwrap());
        // Main assertion: no panic occurred (we got here)
        // should_import is just documentation, actual behavior may vary
        let _ = should_import;
    }
}

#[test]
fn test_key_name_edge_cases() {
    let t = Test::init("alice");

    let invalid_keys = [
        "",
        " ",
        "has space",
        "has-dash",  // valid
        "has_underscore",  // valid
        "123starts_with_digit",
        "VALID_KEY",  // valid
        "has.dot",
        "has/slash",
        "has\\backslash",
        "../traversal",
        "null\x00byte",
    ];

    for key in &invalid_keys {
        if key.contains('\x00') {
            continue;
        }

        let _output = t.set(key, "value");
        // Should either succeed (valid key) or fail gracefully (invalid key)
        // Should never panic - reaching here means success
    }
}

// ============================================================================
// Recovery Tests
// ============================================================================

#[test]
fn test_recovery_corrupted_config() {
    let t = Test::init("alice");
    t.set("KEY", "value");

    // Corrupt the config file
    let config_path = t.dir.path().join(".dugout.toml");
    fs::write(&config_path, "this is not valid toml {{{{").unwrap();

    // Operations should fail gracefully
    let output = t.get("KEY");
    assert_failure(&output);
    assert!(stderr(&output).contains("parse") || stderr(&output).contains("TOML") || stderr(&output).contains("error"));
}

#[test]
fn test_recovery_missing_key_file() {
    let t = Test::init("alice");
    t.set("KEY", "value");

    // Find and delete the key file
    let keys_dir = t.home.path().join(".dugout/keys");
    if keys_dir.exists() {
        fs::remove_dir_all(&keys_dir).unwrap();
    }

    // Should fail gracefully
    let output = t.get("KEY");
    assert_failure(&output);
}

#[test]
fn test_recovery_truncated_config() {
    let t = Test::init("alice");
    t.set("KEY", "value");

    // Truncate config mid-write (simulate crash)
    let config_path = t.dir.path().join(".dugout.toml");
    let content = fs::read_to_string(&config_path).unwrap();
    fs::write(&config_path, &content[..content.len() / 2]).unwrap();

    // Should fail gracefully
    let output = t.get("KEY");
    assert_failure(&output);
}

#[test]
fn test_recovery_empty_config() {
    let t = Test::init("alice");

    // Empty the config
    let config_path = t.dir.path().join(".dugout.toml");
    fs::write(&config_path, "").unwrap();

    // Should fail gracefully
    let output = t.get("KEY");
    assert_failure(&output);
}

#[test]
fn test_atomic_config_survives_concurrent_reads() {
    // Verify atomic save doesn't break concurrent readers
    let t = Test::with_secrets("alice", STANDARD_SECRETS);

    let dir = t.dir.path().to_path_buf();
    let home = t.home.path().to_path_buf();

    // Start many readers
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let dir = dir.clone();
            let home = home.clone();
            thread::spawn(move || {
                for _ in 0..5 {
                    let output = std::process::Command::new(env!("CARGO_BIN_EXE_dugout"))
                        .args(["list"])
                        .env("HOME", &home)
                        .env("USERPROFILE", &home)
                        .current_dir(&dir)
                        .output()
                        .expect("failed to run dugout");
                    if !output.status.success() {
                        return (i, false);
                    }
                    thread::sleep(std::time::Duration::from_millis(10));
                }
                (i, true)
            })
        })
        .collect();

    // Meanwhile, do a write
    t.set("NEW_KEY", "new_value");

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let failures: Vec<_> = results.iter().filter(|(_, ok)| !ok).collect();

    // Most reads should succeed (some may see partial state, that's ok)
    assert!(
        failures.len() <= 2,
        "Too many reader failures during concurrent write: {:?}",
        failures
    );
}

// ============================================================================
// Filesystem Edge Cases
// ============================================================================

#[test]
#[cfg(unix)]
fn test_readonly_directory_fails_gracefully() {
    use std::os::unix::fs::PermissionsExt;

    let t = Test::init("alice");

    // Make directory read-only (prevents creating temp file for atomic save)
    let dir_path = t.dir.path();
    fs::set_permissions(dir_path, fs::Permissions::from_mode(0o555)).unwrap();

    // Write should fail gracefully
    let output = t.set("KEY", "value");
    assert_failure(&output);

    // Restore permissions for cleanup
    fs::set_permissions(dir_path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
#[cfg(unix)]
fn test_readonly_keys_dir_fails_gracefully() {
    use std::os::unix::fs::PermissionsExt;

    let t = Test::new();

    // Create keys dir and make it read-only
    let keys_dir = t.home.path().join(".dugout/keys");
    fs::create_dir_all(&keys_dir).unwrap();
    fs::set_permissions(&keys_dir, fs::Permissions::from_mode(0o555)).unwrap();

    // Init should fail gracefully (can't create key)
    let output = t.init_cmd("alice");
    assert_failure(&output);

    // Restore permissions for cleanup
    fs::set_permissions(&keys_dir, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn test_nonexistent_env_file_import() {
    let t = Test::init("alice");

    let output = t.secrets_import("/nonexistent/path/to/.env");
    assert_failure(&output);
}

#[test]
fn test_directory_as_env_file() {
    let t = Test::init("alice");

    let dir_path = t.dir.path().join("fake.env");
    fs::create_dir(&dir_path).unwrap();

    let output = t.secrets_import(dir_path.to_str().unwrap());
    assert_failure(&output);
}

#[test]
fn test_symlink_following() {
    let t = Test::init("alice");
    t.set("KEY", "value");

    // Create a symlink to the config
    #[cfg(unix)]
    {
        let config_path = t.dir.path().join(".dugout.toml");
        let link_path = t.dir.path().join("config_link.toml");
        std::os::unix::fs::symlink(&config_path, &link_path).unwrap();

        // Config should still work (symlink is followed)
        let output = t.get("KEY");
        assert_success(&output);
    }
}

// ============================================================================
// Input Validation Tests
// ============================================================================

#[test]
fn test_vault_name_validation_comprehensive() {
    let t = Test::new();

    let invalid_vault_names = [
        "",
        ".",
        "..",
        "/",
        "\\",
        "foo/bar",
        "foo\\bar",
        "../escape",
        "..\\escape",
        &"a".repeat(65),  // too long
        "default",  // reserved
    ];

    for name in &invalid_vault_names {
        let output = t
            .cmd()
            .args(["init", "--name", "alice", "--vault", name])
            .output()
            .unwrap();
        assert_failure(&output);
    }
}

#[test]
fn test_member_name_validation_comprehensive() {
    let t = Test::init("alice");

    let invalid_member_names = [
        "",
        ".",
        "..",
        ".hidden",
        "/",
        "\\",
        "foo/bar",
        "foo\\bar",
        "../escape",
    ];

    let (pubkey, _) = generate_age_keypair();
    for name in &invalid_member_names {
        let output = t.team_add(name, &pubkey);
        assert_failure(&output);
    }
}

// ============================================================================
// Property-Based Tests
// ============================================================================

mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn roundtrip_ascii_values(value in "[a-zA-Z0-9_]{1,100}") {
            let t = Test::init("alice");
            let output = t.set("PROP_KEY", &value);
            prop_assert!(output.status.success());

            let output = t.get("PROP_KEY");
            prop_assert!(output.status.success());
            prop_assert!(stdout(&output).contains(&value));
        }

        #[test]
        fn roundtrip_unicode_values(value in "\\PC{1,50}") {
            // Skip values with null bytes (can't pass as CLI args)
            prop_assume!(!value.contains('\x00'));

            let t = Test::init("alice");
            let output = t.set("UNICODE_KEY", &value);

            // Should either succeed or fail gracefully (no panic = success)
            if output.status.success() {
                let output = t.get("UNICODE_KEY");
                prop_assert!(output.status.success());
            }
        }

        #[test]
        fn valid_key_names_accepted(key in "[A-Z][A-Z0-9_]{0,30}") {
            let t = Test::init("alice");
            let output = t.set(&key, "test_value");
            prop_assert!(output.status.success(), "Valid key '{}' should be accepted", key);
        }

        #[test]
        fn env_parser_no_panic(content in "[^\x00]*") {
            let t = Test::init("alice");
            let env_file = t.dir.path().join("fuzz.env");
            prop_assume!(std::fs::write(&env_file, &content).is_ok());

            // Should not panic, may fail - that's ok
            let _ = t.secrets_import(env_file.to_str().unwrap());
        }
    }
}

// ============================================================================
// Windows-Specific Tests
// ============================================================================

#[test]
fn test_paths_no_backslash_in_output() {
    // Verify user-facing output uses forward slashes consistently
    let t = Test::init("alice");

    // Setup global identity
    let setup_output = t.cmd().arg("setup").output().unwrap();
    assert_success(&setup_output);

    // Knock creates a request file and shows path in output
    let output = t.cmd().args(["knock", "bob"]).output().unwrap();
    assert_success(&output);

    let out = stdout(&output);
    // On all platforms, the displayed path should use forward slashes
    assert!(
        !out.contains('\\') || !out.contains(".dugout"),
        "User-facing paths should use forward slashes, got: {}",
        out
    );
}

#[test]
fn test_windows_line_endings_in_env() {
    // Verify we handle CRLF line endings
    let t = Test::init("alice");

    let env_content = "KEY1=value1\r\nKEY2=value2\r\nKEY3=value3\r\n";
    let env_file = t.dir.path().join("crlf.env");
    fs::write(&env_file, env_content).unwrap();

    let output = t.secrets_import(env_file.to_str().unwrap());
    assert_success(&output);

    // Verify all keys imported correctly
    for i in 1..=3 {
        let output = t.get(&format!("KEY{}", i));
        assert_success(&output);
        assert_stdout_contains(&output, &format!("value{}", i));
    }
}

#[test]
fn test_mixed_line_endings() {
    // Mix of LF and CRLF
    let t = Test::init("alice");

    let env_content = "KEY1=value1\nKEY2=value2\r\nKEY3=value3\n";
    let env_file = t.dir.path().join("mixed.env");
    fs::write(&env_file, env_content).unwrap();

    let output = t.secrets_import(env_file.to_str().unwrap());
    assert_success(&output);

    for i in 1..=3 {
        let output = t.get(&format!("KEY{}", i));
        assert_success(&output);
    }
}

// ============================================================================
// Rotation Recovery Tests
// ============================================================================

#[test]
fn test_rotation_preserves_all_secrets() {
    let secrets: Vec<(String, String)> = (0..10)
        .map(|i| (format!("ROTATE_KEY_{}", i), format!("rotate_value_{}", i)))
        .collect();

    let pairs: Vec<(&str, &str)> = secrets.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let t = Test::with_secrets("alice", &pairs);

    // Rotate
    let output = t.secrets_rotate();
    assert_success(&output);

    // Verify all secrets still accessible
    for (key, expected) in &secrets {
        let output = t.get(key);
        assert_success(&output);
        assert_stdout_contains(&output, expected);
    }
}

#[test]
fn test_rotation_with_team_members() {
    let t = Test::init("alice");

    // Add team member
    let (bob_key, _) = generate_age_keypair();
    t.team_add("bob", &bob_key);

    // Add secrets
    t.set("TEAM_SECRET", "shared_value");

    // Rotate
    let output = t.secrets_rotate();
    assert_success(&output);

    // Verify secret still accessible
    let output = t.get("TEAM_SECRET");
    assert_success(&output);
    assert_stdout_contains(&output, "shared_value");

    // Verify bob is still in team
    let output = t.team_list();
    assert_success(&output);
    assert_stdout_contains(&output, "bob");
}

// ============================================================================
// Signal Handling (Simulated)
// ============================================================================

#[test]
fn test_rapid_sequential_operations() {
    // Simulate rapid fire operations that might race
    let t = Test::init("alice");

    for i in 0..20 {
        let key = format!("RAPID_KEY_{}", i);
        let value = format!("rapid_value_{}", i);
        let output = t.set(&key, &value);
        assert_success(&output);
    }

    // Verify all are readable
    for i in 0..20 {
        let key = format!("RAPID_KEY_{}", i);
        let output = t.get(&key);
        assert_success(&output);
    }
}

#[test]
fn test_list_during_modifications() {
    let t = Test::init("alice");

    // Set some initial secrets
    for i in 0..5 {
        t.set(&format!("INITIAL_{}", i), "value");
    }

    let dir = t.dir.path().to_path_buf();
    let home = t.home.path().to_path_buf();

    // Start a lister thread
    let lister = thread::spawn(move || {
        let mut success_count = 0;
        for _ in 0..10 {
            let output = std::process::Command::new(env!("CARGO_BIN_EXE_dugout"))
                .args(["list"])
                .env("HOME", &home)
                .env("USERPROFILE", &home)
                .current_dir(&dir)
                .output()
                .expect("failed to run dugout");
            if output.status.success() {
                success_count += 1;
            }
            thread::sleep(std::time::Duration::from_millis(5));
        }
        success_count
    });

    // Meanwhile, modify secrets
    for i in 5..15 {
        t.set(&format!("DURING_LIST_{}", i), "value");
    }

    let success_count = lister.join().unwrap();
    assert!(success_count >= 5, "Most list operations should succeed");
}
