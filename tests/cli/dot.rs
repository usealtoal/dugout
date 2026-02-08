//! Tests for `burrow .` (dot) command - project detection.

use crate::support::*;
use std::fs;

#[test]
fn test_dot_detects_python() {
    let t = Test::init("alice");

    // Create pyproject.toml
    fs::write(
        t.dir.path().join("pyproject.toml"),
        "[project]\nname = \"test\"\n",
    )
    .unwrap();

    // Set a secret for the command to use
    let set_output = t.set("TEST_VAR", "test_value");
    assert_success(&set_output);

    // Run dot command - it will try to run `uv run`
    let output = t.cmd().arg(".").output().unwrap();

    // Should attempt to run uv, which will fail with pyproject.toml error
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(combined.contains("pyproject"));
}

#[test]
fn test_dot_detects_node() {
    let t = Test::init("alice");

    // Create package.json
    fs::write(t.dir.path().join("package.json"), "{}").unwrap();

    let set_output = t.set("TEST_VAR", "test_value");
    assert_success(&set_output);

    let output = t.cmd().arg(".").output().unwrap();

    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(combined.contains("node") || combined.contains("npm") || combined.contains("bun"));
}

#[test]
fn test_dot_detects_rust() {
    let t = Test::init("alice");

    // Create Cargo.toml
    fs::write(
        t.dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\n",
    )
    .unwrap();

    let set_output = t.set("TEST_VAR", "test_value");
    assert_success(&set_output);

    let output = t.cmd().arg(".").output().unwrap();

    // Should attempt to run cargo, which will fail with Cargo.toml error
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(combined.contains("Cargo"));
}

#[test]
fn test_dot_priority_python_over_node() {
    let t = Test::init("alice");

    // Create both pyproject.toml and package.json
    fs::write(
        t.dir.path().join("pyproject.toml"),
        "[project]\nname = \"test\"\n",
    )
    .unwrap();
    fs::write(t.dir.path().join("package.json"), "{}").unwrap();

    let set_output = t.set("TEST_VAR", "test_value");
    assert_success(&set_output);

    let output = t.cmd().arg(".").output().unwrap();

    // Should prefer Python over Node (pyproject error, not npm)
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(combined.contains("pyproject"));
}

#[test]
fn test_dot_no_project_detected() {
    let t = Test::init("alice");

    // No project files
    let output = t.cmd().arg(".").output().unwrap();

    assert_failure(&output);
    assert_stderr_contains(&output, "couldn't detect project type");
    // Check stdout for the hint
    assert_stdout_contains(&output, "burrow run");
}

#[test]
fn test_dot_without_vault_fails() {
    let t = Test::new();

    // Create a project file
    fs::write(t.dir.path().join("Cargo.toml"), "[package]\n").unwrap();

    let output = t.cmd().arg(".").output().unwrap();

    assert_failure(&output);
    assert_stderr_contains(&output, "no vault found");
}

#[test]
fn test_dot_without_access_suggests_knock() {
    let t = Test::new();

    // Init vault with alice
    let init_output = t.init_cmd("alice");
    assert_success(&init_output);

    // Create project file
    fs::write(t.dir.path().join("Cargo.toml"), "[package]\n").unwrap();

    // Setup different global identity (simulating bob)
    let setup_output = t.cmd().arg("setup").output().unwrap();
    assert_success(&setup_output);

    // Try to run - should suggest knock since bob's identity doesn't match alice's
    let output = t.cmd().arg(".").output().unwrap();

    // This will fail because bob doesn't have access
    assert_failure(&output);
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(combined.contains("knock") || combined.contains("access"));
}

#[test]
fn test_dot_detects_docker_compose() {
    let t = Test::init("alice");

    // Create docker-compose.yml
    fs::write(
        t.dir.path().join("docker-compose.yml"),
        "version: '3'\nservices:\n  test:\n    image: test\n",
    )
    .unwrap();

    let set_output = t.set("TEST_VAR", "test_value");
    assert_success(&set_output);

    let output = t.cmd().arg(".").output().unwrap();

    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(combined.contains("docker"));
}

#[test]
fn test_dot_detects_makefile() {
    let t = Test::init("alice");

    // Create Makefile
    fs::write(t.dir.path().join("Makefile"), "dev:\n\techo test\n").unwrap();

    let set_output = t.set("TEST_VAR", "test_value");
    assert_success(&set_output);

    let output = t.cmd().arg(".").output().unwrap();

    // Should run make dev successfully (outputs "echo test")
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(combined.contains("test") || combined.contains("echo"));
}
