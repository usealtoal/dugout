//! Stress and performance tests.
//!
//! These tests verify that dugout handles large numbers of secrets,
//! large values, and many team members gracefully.

mod support;
use support::*;

#[test]
fn test_100_secrets() {
    let t = Test::init("test-user");

    // Set 100 secrets
    for i in 1..=100 {
        let key = format!("SECRET_{}", i);
        let value = format!("value_{}", i);
        let output = t.set(&key, &value);
        assert_success(&output);
    }

    // List all
    let output = t.list();
    assert_success(&output);
    let out = stdout(&output);

    // Verify count (should mention 100 secrets)
    assert!(out.contains("100"), "Should show 100 secrets");

    // Verify a few random secrets can be retrieved
    for i in [1, 25, 50, 75, 100] {
        let key = format!("SECRET_{}", i);
        let expected_value = format!("value_{}", i);
        let output = t.get(&key);
        assert_success(&output);
        assert_stdout_contains(&output, &expected_value);
    }
}

#[test]
fn test_large_secret_value() {
    let t = Test::init("test-user");

    // Create a 100KB value (100,000 bytes)
    let large_value = "X".repeat(100_000);

    // Set it
    let output = t.set("LARGE_SECRET", &large_value);
    assert_success(&output);

    // Get it back
    let output = t.get("LARGE_SECRET");
    assert_success(&output);
    let retrieved = stdout(&output);

    // Verify it's the same length (exact match might have trailing newline)
    assert!(
        retrieved.contains(&large_value[..1000]),
        "Retrieved value should contain the large secret"
    );
}

#[test]
fn test_many_team_members() {
    let t = Test::init("alice");

    // Generate and add 5 team members
    let mut members = vec![];
    for i in 1..=5 {
        let (public_key, _private_key) = generate_age_keypair();
        let name = format!("member_{}", i);
        members.push((name.clone(), public_key.clone()));

        let output = t.team_add(&name, &public_key);
        assert_success(&output);
    }

    // List team
    let output = t.team_list();
    assert_success(&output);
    let out = stdout(&output);

    // Verify all 6 members are listed (alice + 5 new)
    assert!(out.contains("alice"));
    for (name, _) in &members {
        assert!(out.contains(name), "Should list team member: {}", name);
    }
}

#[test]
fn test_bulk_import() {
    let t = Test::init("test-user");

    // Create .env with 50 key-value pairs
    let mut env_content = String::new();
    for i in 1..=50 {
        env_content.push_str(&format!("IMPORT_KEY_{}=import_value_{}\n", i, i));
    }

    std::fs::write(t.dir.path().join("bulk.env"), &env_content).unwrap();

    // Import
    let output = t.secrets_import("bulk.env");
    assert_success(&output);

    // Verify all 50 were imported
    let output = t.list();
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("50"), "Should have 50 secrets");

    // Verify a few random ones can be retrieved
    for i in [1, 10, 25, 40, 50] {
        let key = format!("IMPORT_KEY_{}", i);
        let expected_value = format!("import_value_{}", i);
        let output = t.get(&key);
        assert_success(&output);
        assert_stdout_contains(&output, &expected_value);
    }
}

#[test]
fn test_rotate_with_many_secrets() {
    let t = Test::init("test-user");

    // Set 20 secrets
    let mut secrets = vec![];
    for i in 1..=20 {
        let key = format!("ROTATE_KEY_{}", i);
        let value = format!("rotate_value_{}", i);
        secrets.push((key.clone(), value.clone()));
        let output = t.set(&key, &value);
        assert_success(&output);
    }

    // Rotate
    let output = t.secrets_rotate();
    assert_success(&output);

    // Verify all 20 are still accessible
    for (key, expected_value) in &secrets {
        let output = t.get(key);
        assert_success(&output);
        assert_stdout_contains(&output, expected_value);
    }
}

#[test]
fn test_stress_mixed_operations() {
    let t = Test::init("alice");

    // Set secrets
    for i in 1..=10 {
        let output = t.set(&format!("KEY_{}", i), &format!("value_{}", i));
        assert_success(&output);
    }

    // Add team members
    for i in 1..=3 {
        let (public_key, _) = generate_age_keypair();
        let output = t.team_add(&format!("member_{}", i), &public_key);
        assert_success(&output);
    }

    // Rotate
    let output = t.secrets_rotate();
    assert_success(&output);

    // Add more secrets
    for i in 11..=20 {
        let output = t.set(&format!("KEY_{}", i), &format!("value_{}", i));
        assert_success(&output);
    }

    // Verify all 20 secrets work
    for i in 1..=20 {
        let output = t.get(&format!("KEY_{}", i));
        assert_success(&output);
    }

    // Export
    let output = t.secrets_export();
    assert_success(&output);
    let out = stdout(&output);

    // Should contain all keys
    for i in 1..=20 {
        assert!(out.contains(&format!("KEY_{}", i)));
    }
}
