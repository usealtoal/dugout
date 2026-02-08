//! Full workflow integration tests.
//!
//! These tests verify complete end-to-end workflows.

mod support;
use support::*;

#[test]
fn test_full_solo_developer_workflow() {
    let t = Test::with_secrets(
        "developer",
        &[
            ("DATABASE_URL", "postgres://localhost/mydb"),
            ("API_KEY", "secret-api-key-12345"),
            ("JWT_SECRET", "super-secret-jwt"),
            ("REDIS_URL", "redis://localhost:6379"),
            ("S3_BUCKET", "my-app-bucket"),
        ],
    );

    // List
    let output = t.list();
    assert_success(&output);
    assert_stdout_contains(&output, "DATABASE_URL");
    assert_stdout_contains(&output, "API_KEY");
    assert_stdout_contains(&output, "JWT_SECRET");
    assert_stdout_contains(&output, "REDIS_URL");
    assert_stdout_contains(&output, "S3_BUCKET");
    assert_stdout_contains(&output, "5");

    // Unlock
    let output = t.secrets_unlock();
    assert_success(&output);

    // Run
    #[cfg(unix)]
    {
        let output = t.run(&["sh", "-c", "echo $DATABASE_URL"]);
        assert_success(&output);
        assert_stdout_contains(&output, "postgres://localhost/mydb");
    }

    // Export
    let output = t.secrets_export();
    assert_success(&output);
    assert_stdout_contains(&output, "DATABASE_URL=postgres://localhost/mydb");
    assert_stdout_contains(&output, "API_KEY=secret-api-key-12345");

    // Import roundtrip
    let test_env_path = t.dir.path().join("backup.env");
    std::fs::write(&test_env_path, "NEW_KEY=new_value\n").unwrap();
    let output = t.secrets_import("backup.env");
    assert_success(&output);

    let output = t.get("NEW_KEY");
    assert_success(&output);
    assert_stdout_contains(&output, "new_value");
}

#[test]
fn test_full_team_workflow() {
    let t = Test::with_secrets(
        "alice",
        &[
            ("TEAM_DATABASE", "postgres://team/db"),
            ("TEAM_API_KEY", "team-secret"),
            ("SHARED_CONFIG", "config-value"),
        ],
    );

    // Add team member
    let output = t.team_add("bob", BOB_PUBLIC_KEY);
    assert_success(&output);

    // Team list
    let output = t.team_list();
    assert_success(&output);
    assert_stdout_contains(&output, "alice");
    assert_stdout_contains(&output, "bob");

    // Rotate
    let output = t.secrets_rotate();
    assert_success(&output);

    // Verify secrets still accessible after rotation
    let output = t.get("TEAM_DATABASE");
    assert_success(&output);
    assert_stdout_contains(&output, "postgres://team/db");

    let output = t.get("TEAM_API_KEY");
    assert_success(&output);
    assert_stdout_contains(&output, "team-secret");

    let output = t.get("SHARED_CONFIG");
    assert_success(&output);
    assert_stdout_contains(&output, "config-value");
}

#[test]
fn test_rotation_preserves_all_secrets() {
    let secrets = [
        ("SECRET_1", "value_1"),
        ("SECRET_2", "value_2"),
        ("SECRET_3", "value_3"),
        ("SECRET_4", "value_4"),
        ("SECRET_5", "value_5"),
        ("SECRET_6", "value_6"),
        ("SECRET_7", "value_7"),
        ("SECRET_8", "value_8"),
        ("SECRET_9", "value_9"),
        ("SECRET_10", "value_10"),
    ];

    let t = Test::with_secrets("test-user", &secrets);

    // Rotate
    let output = t.secrets_rotate();
    assert_success(&output);

    // Verify all 10 still decrypt correctly
    for (key, expected_val) in &secrets {
        let output = t.get(key);
        assert_success(&output);
        assert_stdout_contains(&output, expected_val);
    }
}

#[test]
fn test_complex_env_import_workflow() {
    let t = Test::init("test-user");

    // Import complex .env file
    let test_env = t.dir.path().join("complex.env");
    std::fs::write(&test_env, SAMPLE_ENV_COMPLEX).unwrap();

    let output = t.secrets_import("complex.env");
    assert_success(&output);

    // Verify keys were imported correctly
    let output = t.get("SIMPLE");
    assert_success(&output);
    assert_stdout_contains(&output, "value");

    let output = t.get("QUOTED");
    assert_success(&output);
    assert_stdout_contains(&output, "quoted value");
}

#[test]
fn test_standard_secrets_roundtrip() {
    let t = Test::with_secrets("test-user", STANDARD_SECRETS);

    // Export
    let output = t.secrets_export();
    assert_success(&output);
    let exported = stdout(&output);

    // Verify all standard secrets are present
    for (key, _) in STANDARD_SECRETS {
        assert!(exported.contains(key), "Missing key: {}", key);
    }
}
