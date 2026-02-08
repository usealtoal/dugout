//! Full workflow integration tests.
//!
//! These tests verify complete end-to-end workflows.

mod harness;
use harness::{assert_success, stdout, TestEnv};

#[test]
fn test_full_solo_developer_workflow() {
    let env = TestEnv::new();

    // init
    let output = env.init("developer");
    assert_success(&output);

    // set 5 secrets
    env.set("DATABASE_URL", "postgres://localhost/mydb");
    env.set("API_KEY", "secret-api-key-12345");
    env.set("JWT_SECRET", "super-secret-jwt");
    env.set("REDIS_URL", "redis://localhost:6379");
    env.set("S3_BUCKET", "my-app-bucket");

    // list
    let output = env.list();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("DATABASE_URL"));
    assert!(out.contains("API_KEY"));
    assert!(out.contains("JWT_SECRET"));
    assert!(out.contains("REDIS_URL"));
    assert!(out.contains("S3_BUCKET"));
    assert!(out.contains("5"));

    // unlock
    let output = env.secrets_unlock();
    assert_success(&output);

    // run
    #[cfg(unix)]
    {
        let output = env.run(&["sh", "-c", "echo $DATABASE_URL"]);
        assert_success(&output);
        let out = stdout(&output);
        assert!(out.contains("postgres://localhost/mydb"));
    }

    // export
    let output = env.secrets_export();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("DATABASE_URL=postgres://localhost/mydb"));
    assert!(out.contains("API_KEY=secret-api-key-12345"));

    // import roundtrip
    let test_env_path = env.dir.path().join("backup.env");
    std::fs::write(&test_env_path, "NEW_KEY=new_value\n").unwrap();
    let output = env.secrets_import("backup.env");
    assert_success(&output);

    let output = env.get("NEW_KEY");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("new_value"));
}

#[test]
fn test_full_team_workflow() {
    let env = TestEnv::new();

    // init
    let output = env.init("alice");
    assert_success(&output);

    // set secrets
    env.set("TEAM_DATABASE", "postgres://team/db");
    env.set("TEAM_API_KEY", "team-secret");
    env.set("SHARED_CONFIG", "config-value");

    // add team member
    let bob_key = "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p";
    let output = env.team_add("bob", bob_key);
    assert_success(&output);

    // team list
    let output = env.team_list();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("alice"));
    assert!(out.contains("bob"));

    // rotate
    let output = env.secrets_rotate();
    assert_success(&output);

    // verify secrets still accessible after rotation
    let output = env.get("TEAM_DATABASE");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("postgres://team/db"));

    let output = env.get("TEAM_API_KEY");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("team-secret"));

    let output = env.get("SHARED_CONFIG");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("config-value"));
}

#[test]
fn test_rotation_preserves_all_secrets() {
    let env = TestEnv::new();

    // init
    let output = env.init("test-user");
    assert_success(&output);

    // set 10 secrets
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

    for (key, val) in &secrets {
        env.set(key, val);
    }

    // rotate
    let output = env.secrets_rotate();
    assert_success(&output);

    // verify all 10 still decrypt correctly
    for (key, expected_val) in &secrets {
        let output = env.get(key);
        assert_success(&output);
        let out = stdout(&output);
        assert!(
            out.contains(expected_val),
            "Failed to decrypt {} after rotation",
            key
        );
    }
}
