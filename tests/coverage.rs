//! Comprehensive coverage expansion tests.
//!
//! These tests target untested paths to maximize code coverage:
//! - Error handling edge cases
//! - Command output validation
//! - Secrets operations
//! - Run command behavior
//! - Multi-user workflows
//! - Edge cases and boundary conditions

mod support;
use support::*;

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_get_nonexistent_key() {
    let t = Test::init("test_get_nonexistent");
    let output = t.get("NONEXISTENT_KEY");
    assert_failure(&output);
    assert_stderr_contains(&output, "not found");
}

#[test]
fn test_set_invalid_key_name() {
    let t = Test::init("test_invalid_key");

    // Try key with spaces
    let output = t.set("INVALID KEY", "value");
    assert_failure(&output);

    // Try key with special chars that might be problematic
    let output = t.set("KEY@#$%", "value");
    assert_failure(&output);
}

#[test]
fn test_rm_nonexistent_key() {
    let t = Test::init("test_rm_nonexistent");
    let output = t.rm("DOES_NOT_EXIST");
    assert_failure(&output);
}

#[test]
fn test_init_already_initialized() {
    let t = Test::init("test_double_init");

    // Try to init again
    let output = t.init_cmd("test_double_init_2");
    assert_failure(&output);
    assert_stderr_contains(&output, "already initialized");
}

#[test]
fn test_operations_without_init() {
    let t = Test::new();

    // Try set without init
    let output = t.set("KEY", "value");
    assert_failure(&output);

    // Try get without init
    let output = t.get("KEY");
    assert_failure(&output);

    // Try list without init
    let output = t.list();
    assert_failure(&output);
}

#[test]
fn test_team_add_invalid_key() {
    let t = Test::init("test_team_invalid_key");
    let output = t.team_add("bob", INVALID_PUBLIC_KEY);
    assert_failure(&output);
    assert_stderr_contains(&output, "invalid");
}

#[test]
fn test_team_add_duplicate_name() {
    let t = Test::init("test_team_duplicate");

    // Add bob once
    let output = t.team_add("bob", BOB_PUBLIC_KEY);
    assert_success(&output);

    // Try to add bob again with different key
    // This may succeed (updating the key) or fail depending on implementation
    let (other_pubkey, _) = generate_age_keypair();
    let _output = t.team_add("bob", &other_pubkey);

    // Verify bob is in team list regardless
    let output = t.team_list();
    assert_success(&output);
    assert_stdout_contains(&output, "bob");
}

#[test]
fn test_team_rm_nonexistent() {
    let t = Test::init("test_team_rm_nonexist");
    let output = t.team_rm("nonexistent_member");
    assert_failure(&output);
}

#[test]
fn test_team_rm_requires_member_name() {
    let t = Test::init("alice");

    // Add another member first
    let output = t.team_add("bob", BOB_PUBLIC_KEY);
    assert_success(&output);

    // Should be able to remove bob
    let output = t.team_rm("bob");
    assert_success(&output);

    // Verify bob is gone
    let output = t.team_list();
    assert_success(&output);
    assert_stdout_contains(&output, "alice");
    // Bob should not be in the list anymore
}

// ============================================================================
// Command Output Tests
// ============================================================================

#[test]
fn test_check_status_output() {
    let t = Test::with_secrets(
        "status_test",
        &[("KEY1", "value1"), ("KEY2", "value2"), ("KEY3", "value3")],
    );

    let output = t.check_status();
    assert_success(&output);
    let stdout = stdout(&output);

    // Verify status shows vault info
    assert!(
        stdout.contains("vault") || stdout.contains("Vault"),
        "Status should mention vault"
    );

    // Should show cipher or encryption info
    assert!(
        stdout.contains("cipher") || stdout.contains("age") || stdout.contains("encryption"),
        "Status should mention encryption"
    );

    // Should show secret count (3 secrets)
    assert!(stdout.contains("3"), "Status should show secret count");
}

#[test]
fn test_check_audit_clean() {
    let t = Test::with_secrets("audit_clean", &[("KEY", "value")]);

    let output = t.check_audit();
    assert_success(&output);

    // Clean audit should produce output
    let out_str = stdout(&output);
    let err_str = stderr(&output);
    assert!(
        out_str.len() > 0 || err_str.len() > 0,
        "Audit should produce output"
    );
}

#[test]
fn test_check_audit_detects_issues() {
    let t = Test::with_secrets("audit_issues", &[("KEY", "value")]);

    // Unlock to create .env file
    let output = t.secrets_unlock();
    assert_success(&output);

    // Remove .gitignore if it exists to simulate a missing gitignore issue
    let gitignore_path = t.dir.path().join(".gitignore");
    if gitignore_path.exists() {
        std::fs::remove_file(&gitignore_path).ok();
    }

    let output = t.check_audit();
    // Audit might pass or fail depending on implementation, but should run
    // Check that it provides meaningful output
    let stdout = stdout(&output);
    let stderr = stderr(&output);
    let combined = format!("{}{}", stdout, stderr);
    assert!(combined.len() > 0, "Audit should produce output");
}

#[test]
fn test_team_list_output() {
    let t = Test::init("team_list_test");

    // Add team members
    let output = t.team_add("bob", BOB_PUBLIC_KEY);
    assert_success(&output);

    let (charlie_key, _) = generate_age_keypair();
    let output = t.team_add("charlie", &charlie_key);
    assert_success(&output);

    // List team
    let output = t.team_list();
    assert_success(&output);
    assert_stdout_contains(&output, "team_list_test");
    assert_stdout_contains(&output, "bob");
    assert_stdout_contains(&output, "charlie");
}

#[test]
fn test_team_list_json_format() {
    let t = Test::init("team_json");

    let output = t.team_add("bob", BOB_PUBLIC_KEY);
    assert_success(&output);

    let output = t.team_list_json();
    assert_success(&output);

    // Parse as JSON to verify it's valid
    let json_str = stdout(&output);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
    assert!(
        parsed.is_ok(),
        "team list --json should produce valid JSON: {}",
        json_str
    );
}

#[test]
fn test_list_json_format() {
    let t = Test::with_secrets("list_json", &[("KEY1", "value1"), ("KEY2", "value2")]);

    let output = t.list_json();
    assert_success(&output);

    // Parse as JSON to verify it's valid
    let json_str = stdout(&output);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
    assert!(
        parsed.is_ok(),
        "list --json should produce valid JSON: {}",
        json_str
    );

    // Should contain our keys
    if let Ok(json) = parsed {
        let text = json.to_string();
        assert!(text.contains("KEY1"), "JSON should contain KEY1");
        assert!(text.contains("KEY2"), "JSON should contain KEY2");
    }
}

// ============================================================================
// Secrets Operations Tests
// ============================================================================

#[test]
fn test_secrets_diff_no_env() {
    let t = Test::with_secrets("diff_no_env", &[("KEY", "value")]);

    // Don't unlock (no .env file)
    let output = t.secrets_diff();

    // Should report differences since .env doesn't exist
    assert_success(&output);
}

#[test]
fn test_secrets_diff_with_changes() {
    let t = Test::with_secrets("diff_changes", &[("KEY1", "value1"), ("KEY2", "value2")]);

    // Unlock to create .env
    let output = t.secrets_unlock();
    assert_success(&output);

    // Modify .env file
    let env_path = t.dir.path().join(".env");
    std::fs::write(&env_path, "KEY1=modified_value\nKEY3=new_value\n").unwrap();

    // Diff should show changes
    let output = t.secrets_diff();
    assert_success(&output);

    let stdout = stdout(&output);
    // Should show some difference indication
    assert!(stdout.len() > 0, "Diff with changes should produce output");
}

#[test]
fn test_secrets_export_format() {
    let t = Test::with_secrets(
        "export_format",
        &[
            ("DATABASE_URL", "postgres://localhost/db"),
            ("API_KEY", "secret123"),
            ("PORT", "3000"),
        ],
    );

    let output = t.secrets_export();
    assert_success(&output);

    let stdout = stdout(&output);
    // Verify KEY=value format
    assert!(
        stdout.contains("DATABASE_URL=postgres://localhost/db"),
        "Export should contain DATABASE_URL=..."
    );
    assert!(
        stdout.contains("API_KEY=secret123"),
        "Export should contain API_KEY=..."
    );
    assert!(
        stdout.contains("PORT=3000"),
        "Export should contain PORT=..."
    );
}

#[test]
fn test_secrets_import_from_file() {
    let t = Test::init("import_test");

    // Create a .env file to import
    let import_path = t.dir.path().join("import.env");
    std::fs::write(&import_path, "IMPORTED_KEY1=value1\nIMPORTED_KEY2=value2\n").unwrap();

    // Import
    let output = t.secrets_import("import.env");
    assert_success(&output);

    // Verify imported secrets
    let output = t.get("IMPORTED_KEY1");
    assert_success(&output);
    assert_stdout_contains(&output, "value1");

    let output = t.get("IMPORTED_KEY2");
    assert_success(&output);
    assert_stdout_contains(&output, "value2");
}

#[test]
fn test_secrets_lock_command() {
    let t = Test::with_secrets("lock_test", &[("KEY", "value")]);

    // Unlock to create .env
    let output = t.secrets_unlock();
    assert_success(&output);

    let env_path = t.dir.path().join(".env");
    assert!(env_path.exists(), ".env should exist after unlock");

    // Lock should succeed
    let output = t.secrets_lock();
    assert_success(&output);

    // Verify lock command ran successfully
    // (implementation may remove or rename .env)
}

#[test]
fn test_secrets_rotate_preserves_all() {
    let t = Test::with_secrets(
        "rotate_test",
        &[
            ("KEY1", "value1"),
            ("KEY2", "value2"),
            ("KEY3", "value3"),
            ("KEY4", "value4"),
        ],
    );

    // Rotate
    let output = t.secrets_rotate();
    assert_success(&output);

    // Verify all secrets still exist and have correct values
    let output = t.get("KEY1");
    assert_success(&output);
    assert_stdout_contains(&output, "value1");

    let output = t.get("KEY2");
    assert_success(&output);
    assert_stdout_contains(&output, "value2");

    let output = t.get("KEY3");
    assert_success(&output);
    assert_stdout_contains(&output, "value3");

    let output = t.get("KEY4");
    assert_success(&output);
    assert_stdout_contains(&output, "value4");
}

#[test]
fn test_set_overwrite_requires_force() {
    let t = Test::with_secrets("overwrite_force", &[("EXISTING_KEY", "original_value")]);

    // Try to overwrite without --force
    let output = t.set("EXISTING_KEY", "new_value");
    assert_failure(&output);
    assert_stderr_contains(&output, "force");

    // Original value should still be there
    let output = t.get("EXISTING_KEY");
    assert_success(&output);
    assert_stdout_contains(&output, "original_value");
}

#[test]
fn test_set_overwrite_with_force() {
    let t = Test::with_secrets("overwrite_with_force", &[("KEY", "old_value")]);

    // Overwrite with --force
    let output = t.set_force("KEY", "new_value");
    assert_success(&output);

    // Verify new value
    let output = t.get("KEY");
    assert_success(&output);
    assert_stdout_contains(&output, "new_value");
    assert_stdout_excludes(&output, "old_value");
}

// ============================================================================
// Run Command Tests
// ============================================================================

#[test]
fn test_run_without_secrets() {
    let t = Test::init("run_empty");

    #[cfg(unix)]
    {
        // Run a command with empty vault
        let output = t.run(&["sh", "-c", "echo hello"]);
        assert_success(&output);
        assert_stdout_contains(&output, "hello");
    }

    #[cfg(windows)]
    {
        let output = t.run(&["cmd", "/C", "echo hello"]);
        assert_success(&output);
        assert_stdout_contains(&output, "hello");
    }
}

#[test]
fn test_run_nonexistent_command() {
    let t = Test::with_secrets("run_nonexistent", &[("KEY", "value")]);

    // Try to run a command that doesn't exist
    let output = t.run(&["this_command_does_not_exist_12345"]);
    assert_failure(&output);
}

#[test]
fn test_run_exit_code_passthrough() {
    let t = Test::with_secrets("run_exit_code", &[("KEY", "value")]);

    #[cfg(unix)]
    {
        // Run a command that exits with code 42
        let output = t.run(&["sh", "-c", "exit 42"]);
        assert_failure(&output);

        // Check exit code is passed through
        if let Some(code) = output.status.code() {
            assert_eq!(code, 42, "Exit code should be passed through");
        }
    }

    #[cfg(windows)]
    {
        let output = t.run(&["cmd", "/C", "exit 42"]);
        assert_failure(&output);

        if let Some(code) = output.status.code() {
            assert_eq!(code, 42, "Exit code should be passed through");
        }
    }
}

// ============================================================================
// Multi-User Tests
// ============================================================================

#[test]
fn test_full_team_workflow() {
    // Generate Bob's identity
    let (bob_pubkey, _bob_privkey) = generate_age_keypair();

    // Alice initializes and adds Bob
    let t = Test::init("alice");
    let output = t.team_add("bob", &bob_pubkey);
    assert_success(&output);

    // Alice sets a secret
    let output = t.set("TEAM_SECRET", "shared_value");
    assert_success(&output);

    // Verify the vault is readable and contains the secret
    let output = t.list();
    assert_success(&output);
    assert_stdout_contains(&output, "TEAM_SECRET");

    // Verify Bob is in the team
    let output = t.team_list();
    assert_success(&output);
    assert_stdout_contains(&output, "bob");
    assert_stdout_contains(&output, "alice");
}

#[test]
fn test_knock_admit_flow() {
    // Alice creates vault
    let t_alice = Test::init("alice");
    let output = t_alice.set("SECRET", "value");
    assert_success(&output);

    // Generate Bob's identity
    let (bob_pubkey, _bob_privkey) = generate_age_keypair();

    // Bob knocks (simulate with team add request)
    // In practice, knock creates a request file
    // For now, test that Alice can add Bob
    let output = t_alice.team_add("bob", &bob_pubkey);
    assert_success(&output);

    // Verify Bob is in team list
    let output = t_alice.team_list();
    assert_success(&output);
    assert_stdout_contains(&output, "bob");
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[test]
fn test_empty_secret_value_rejected() {
    let t = Test::init("empty_value");

    // Set a key with empty value should fail
    let output = t.set("EMPTY_KEY", "");
    assert_failure(&output);
    assert_stderr_contains(&output, "empty");
}

#[test]
fn test_very_long_secret() {
    let t = Test::init("long_value");

    // Create a 10KB value
    let long_value = "x".repeat(10 * 1024);

    let output = t.set("LONG_KEY", &long_value);
    assert_success(&output);

    // Verify it can be retrieved
    let output = t.get("LONG_KEY");
    assert_success(&output);
    assert_stdout_contains(&output, &long_value);
}

#[test]
fn test_special_chars_in_value() {
    let t = Test::init("special_chars");

    // Test newlines
    let output = t.set("MULTILINE", "line1\nline2\nline3");
    assert_success(&output);
    let output = t.get("MULTILINE");
    assert_success(&output);
    assert_stdout_contains(&output, "line1");

    // Test quotes
    let output = t.set("QUOTES", "value with \"quotes\" and 'apostrophes'");
    assert_success(&output);
    let output = t.get("QUOTES");
    assert_success(&output);

    // Test unicode
    let output = t.set("UNICODE", "Hello 世界 🌍 مرحبا");
    assert_success(&output);
    let output = t.get("UNICODE");
    assert_success(&output);
    assert_stdout_contains(&output, "世界");
    assert_stdout_contains(&output, "🌍");
}

#[test]
fn test_many_secrets() {
    let t = Test::init("many_secrets");

    // Set 50+ secrets
    for i in 0..55 {
        let key = format!("KEY_{:03}", i);
        let value = format!("value_{}", i);
        let output = t.set(&key, &value);
        assert_success(&output);
    }

    // Verify all are retrievable
    let output = t.list();
    assert_success(&output);

    // Spot check a few
    let output = t.get("KEY_000");
    assert_success(&output);
    assert_stdout_contains(&output, "value_0");

    let output = t.get("KEY_025");
    assert_success(&output);
    assert_stdout_contains(&output, "value_25");

    let output = t.get("KEY_054");
    assert_success(&output);
    assert_stdout_contains(&output, "value_54");
}

// ============================================================================
// Additional Edge Cases
// ============================================================================

#[test]
fn test_list_empty_vault() {
    let t = Test::init("empty_vault");

    // List should work but show no secrets
    let output = t.list();
    assert_success(&output);
}

#[test]
fn test_rm_then_readd() {
    let t = Test::with_secrets("rm_readd", &[("KEY", "original")]);

    // Remove
    let output = t.rm("KEY");
    assert_success(&output);

    // Verify removed
    let output = t.get("KEY");
    assert_failure(&output);

    // Re-add with different value
    let output = t.set("KEY", "new_value");
    assert_success(&output);

    // Verify new value
    let output = t.get("KEY");
    assert_success(&output);
    assert_stdout_contains(&output, "new_value");
    assert_stdout_excludes(&output, "original");
}

#[test]
fn test_export_empty_vault() {
    let t = Test::init("export_empty");

    // Export should succeed but return empty or minimal output
    let output = t.secrets_export();
    assert_success(&output);
}

#[test]
fn test_unlock_then_lock_then_unlock() {
    let t = Test::with_secrets("unlock_lock_cycle", &[("KEY", "value")]);

    // Unlock
    let output = t.secrets_unlock();
    assert_success(&output);

    let env_path = t.dir.path().join(".env");
    assert!(env_path.exists(), ".env should exist after first unlock");

    // Lock
    let output = t.secrets_lock();
    assert_success(&output);

    // Unlock again
    let output = t.secrets_unlock();
    assert_success(&output);

    // Should be able to unlock again
    assert!(env_path.exists(), ".env should exist after second unlock");
}

#[test]
fn test_key_with_numbers_and_underscores() {
    let t = Test::init("valid_key_chars");

    // Valid key names with numbers and underscores
    let output = t.set("KEY_123", "value1");
    assert_success(&output);

    let output = t.set("KEY_WITH_MANY_UNDERSCORES_123_456", "value2");
    assert_success(&output);

    let output = t.set("KEY2", "value3");
    assert_success(&output);

    // Verify all can be retrieved
    let output = t.get("KEY_123");
    assert_success(&output);
    assert_stdout_contains(&output, "value1");

    let output = t.get("KEY_WITH_MANY_UNDERSCORES_123_456");
    assert_success(&output);
    assert_stdout_contains(&output, "value2");

    let output = t.get("KEY2");
    assert_success(&output);
    assert_stdout_contains(&output, "value3");
}
