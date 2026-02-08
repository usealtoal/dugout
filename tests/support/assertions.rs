//! Test assertion helpers.

use std::process::Output;

/// Assert that a command output was successful.
pub fn assert_success(output: &Output) {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Command failed:\n{}", stderr);
    }
}

/// Assert that a command output failed.
pub fn assert_failure(output: &Output) {
    assert!(
        !output.status.success(),
        "Expected command to fail but it succeeded"
    );
}

/// Get stdout as String.
pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Get stderr as String.
pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// Assert stdout contains a string.
pub fn assert_stdout_contains(output: &Output, expected: &str) {
    let out = stdout(output);
    assert!(
        out.contains(expected),
        "stdout missing '{}', got: {}",
        expected,
        out
    );
}

/// Assert stderr contains a string.
pub fn assert_stderr_contains(output: &Output, expected: &str) {
    let err = stderr(output);
    assert!(
        err.contains(expected),
        "stderr missing '{}', got: {}",
        expected,
        err
    );
}

/// Assert stdout does NOT contain a string.
pub fn assert_stdout_excludes(output: &Output, excluded: &str) {
    let out = stdout(output);
    assert!(
        !out.contains(excluded),
        "stdout should not contain '{}', got: {}",
        excluded,
        out
    );
}

/// Assert set/get roundtrip works for a key-value pair.
pub fn assert_roundtrip(t: &super::Test, key: &str, value: &str) {
    let output = t.set(key, value);
    assert_success(&output);

    let output = t.get(key);
    assert_success(&output);
    assert_stdout_contains(&output, value);
}
