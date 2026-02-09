//! Tests for `dugout .` (dot) command - project detection.
//!
//! These tests must run serially because the test harness uses `set_current_dir`,
//! which is process-global and causes races when tests run in parallel.

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

    // Run dot command - should detect python and attempt to run
    // Won't detect as "no project" or suggest `dugout run`
    let output = t.cmd().arg(".").output().unwrap();
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        !combined.contains("couldn't detect project type"),
        "should detect python project, got: {combined}"
    );
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
    assert!(
        !combined.contains("couldn't detect project type"),
        "should detect node project, got: {combined}"
    );
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

    // Should prefer Python over Node (no npm/bun in output)
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        !combined.contains("couldn't detect project type"),
        "should detect a project type, got: {combined}"
    );
    assert!(
        !combined.contains("npm") && !combined.contains("bun"),
        "should prefer python over node, got: {combined}"
    );
}

#[test]
fn test_dot_no_project_detected() {
    let t = Test::init("alice");

    // No project files
    let output = t.cmd().arg(".").output().unwrap();

    assert_failure(&output);
    assert_stderr_contains(&output, "couldn't detect project type");
    // Check stdout for the hint
    assert_stdout_contains(&output, "dugout run");
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

#[test]
fn test_dot_detects_deno() {
    let t = Test::init("alice");

    fs::write(t.dir.path().join("deno.json"), "{}").unwrap();

    let set_output = t.set("TEST_VAR", "test_value");
    assert_success(&set_output);

    let output = t.cmd().arg(".").output().unwrap();
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        !combined.contains("couldn't detect project type"),
        "should detect deno project, got: {combined}"
    );
}

#[test]
fn test_dot_detects_deno_jsonc() {
    let t = Test::init("alice");

    fs::write(t.dir.path().join("deno.jsonc"), "{}").unwrap();

    let set_output = t.set("TEST_VAR", "test_value");
    assert_success(&set_output);

    let output = t.cmd().arg(".").output().unwrap();
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        !combined.contains("couldn't detect project type"),
        "should detect deno project from deno.jsonc, got: {combined}"
    );
}

#[test]
fn test_dot_deno_priority_over_node() {
    let t = Test::init("alice");

    // Deno should win over Node
    fs::write(t.dir.path().join("deno.json"), "{}").unwrap();
    fs::write(t.dir.path().join("package.json"), "{}").unwrap();

    let set_output = t.set("TEST_VAR", "test_value");
    assert_success(&set_output);

    let output = t.cmd().arg(".").output().unwrap();
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    // Should not run npm/bun (node tools)
    assert!(
        !combined.contains("npm") && !combined.contains("bun"),
        "should prefer deno over node, got: {combined}"
    );
}

#[test]
fn test_dot_detects_ruby() {
    let t = Test::init("alice");

    fs::write(
        t.dir.path().join("Gemfile"),
        "source 'https://rubygems.org'\n",
    )
    .unwrap();

    let set_output = t.set("TEST_VAR", "test_value");
    assert_success(&set_output);

    let output = t.cmd().arg(".").output().unwrap();
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        !combined.contains("couldn't detect project type"),
        "should detect ruby project, got: {combined}"
    );
}

#[test]
fn test_dot_detects_elixir() {
    let t = Test::init("alice");

    fs::write(
        t.dir.path().join("mix.exs"),
        "defmodule MyApp.MixProject do\nend\n",
    )
    .unwrap();

    let set_output = t.set("TEST_VAR", "test_value");
    assert_success(&set_output);

    let output = t.cmd().arg(".").output().unwrap();
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        !combined.contains("couldn't detect project type"),
        "should detect elixir project, got: {combined}"
    );
}

#[test]
fn test_dot_python_no_entry_shows_summary() {
    let t = Test::init("alice");

    // pyproject.toml without any entry points
    fs::write(
        t.dir.path().join("pyproject.toml"),
        "[project]\nname = \"mylib\"\n",
    )
    .unwrap();

    let set_output = t.set("DB_URL", "postgres://localhost");
    assert_success(&set_output);

    let output = t.cmd().arg(".").output().unwrap();
    let combined = format!("{}{}", stdout(&output), stderr(&output));

    // Should show the detection summary line
    assert!(
        combined.contains("python") && combined.contains("secret"),
        "should show summary with project type and secret count, got: {combined}"
    );
}

#[test]
fn test_dot_python_with_main_py() {
    let t = Test::init("alice");

    fs::write(
        t.dir.path().join("pyproject.toml"),
        "[project]\nname = \"myapp\"\n",
    )
    .unwrap();
    fs::write(
        t.dir.path().join("main.py"),
        "import os\nprint(os.environ.get('TEST_SECRET', 'missing'))\n",
    )
    .unwrap();

    let set_output = t.set("TEST_SECRET", "hunter2");
    assert_success(&set_output);

    let output = t.cmd().arg(".").output().unwrap();
    let combined = format!("{}{}", stdout(&output), stderr(&output));

    // Should detect main.py and run it with secrets
    assert!(
        combined.contains("hunter2") || combined.contains("main.py"),
        "should run main.py with injected secrets, got: {combined}"
    );
}

#[test]
fn test_dot_detects_justfile() {
    let t = Test::init("alice");

    fs::write(t.dir.path().join("justfile"), "dev:\n  echo test-just\n").unwrap();

    let set_output = t.set("TEST_VAR", "test_value");
    assert_success(&set_output);

    let output = t.cmd().arg(".").output().unwrap();
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        !combined.contains("couldn't detect project type"),
        "should detect justfile project, got: {combined}"
    );
}
