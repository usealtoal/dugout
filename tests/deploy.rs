//! Deployment and CI/CD scenario tests.
//!
//! Verifies that dugout works with environment-injected identities
//! for CI runners and production servers.

mod support;
use support::*;

// --- DUGOUT_IDENTITY env var ---

#[test]
fn test_decrypt_via_env_identity() {
    let t = Test::init("alice");
    t.set("API_KEY", "sk_live_abc123");

    let identity_content = find_identity_key(&t);
    remove_all_identities(&t);

    let output = t
        .cmd()
        .env("DUGOUT_IDENTITY", identity_content.trim())
        .args(["get", "API_KEY"])
        .output()
        .unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "sk_live_abc123");
}

#[test]
fn test_run_via_env_identity() {
    let t = Test::init("alice");
    t.set("MY_SECRET", "hunter2");

    let identity_content = find_identity_key(&t);
    remove_all_identities(&t);

    let output = t
        .cmd()
        .env("DUGOUT_IDENTITY", identity_content.trim())
        .args(["run", "--", "printenv", "MY_SECRET"])
        .output()
        .unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "hunter2");
}

#[test]
fn test_list_via_env_identity() {
    let t = Test::init("alice");
    t.set("SECRET_A", "aaa");
    t.set("SECRET_B", "bbb");

    let identity_content = find_identity_key(&t);
    remove_all_identities(&t);

    let output = t
        .cmd()
        .env("DUGOUT_IDENTITY", identity_content.trim())
        .args(["list"])
        .output()
        .unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "SECRET_A");
    assert_stdout_contains(&output, "SECRET_B");
}

// --- DUGOUT_IDENTITY_FILE env var ---

#[test]
fn test_decrypt_via_env_identity_file() {
    let t = Test::init("alice");
    t.set("DB_URL", "postgres://localhost/prod");

    let identity_content = find_identity_key(&t);
    let ci_key_path = t.dir.path().join("ci-identity.key");
    std::fs::write(&ci_key_path, &identity_content).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&ci_key_path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }

    remove_all_identities(&t);

    let output = t
        .cmd()
        .env("DUGOUT_IDENTITY_FILE", ci_key_path.to_str().unwrap())
        .args(["get", "DB_URL"])
        .output()
        .unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "postgres://localhost/prod");
}

// --- setup --output ---

#[test]
fn test_setup_output_to_stdout() {
    let t = Test::new();

    let output = t.cmd().args(["setup", "--output", "-"]).output().unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "AGE-SECRET-KEY-");
}

#[test]
fn test_setup_output_to_file() {
    let t = Test::new();

    let key_path = t.dir.path().join("ci.key");
    let output = t
        .cmd()
        .args(["setup", "--output", key_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&output);

    let contents = std::fs::read_to_string(&key_path).unwrap();
    assert!(
        contents.contains("AGE-SECRET-KEY-"),
        "File should contain private key"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&key_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "Key file should have 0600 permissions");
    }
}

#[test]
fn test_setup_with_name_flag() {
    let t = Test::new();

    let output = t
        .cmd()
        .args(["setup", "--name", "ci-runner"])
        .output()
        .unwrap();
    assert_success(&output);

    let output = t.cmd().arg("whoami").output().unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "age1");
}

// --- Error cases ---

#[test]
fn test_no_identity_gives_error() {
    let t = Test::init("alice");
    t.set("SECRET", "value");

    remove_all_identities(&t);

    let output = t.cmd().args(["get", "SECRET"]).output().unwrap();
    assert_failure(&output);
}

#[test]
fn test_invalid_env_identity_falls_through() {
    let t = Test::init("alice");
    t.set("SECRET", "value");

    // Invalid key in env — should fall through to filesystem identity
    let output = t
        .cmd()
        .env("DUGOUT_IDENTITY", "not-a-valid-key")
        .args(["get", "SECRET"])
        .output()
        .unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "value");
}

#[test]
fn test_env_identity_wrong_key_denied() {
    let t = Test::init("alice");
    t.set("SECRET", "value");

    // Generate a different identity not in the recipients
    let other = age::x25519::Identity::generate();
    use age::secrecy::ExposeSecret;
    let other_key = other.to_string();

    remove_all_identities(&t);

    let output = t
        .cmd()
        .env("DUGOUT_IDENTITY", other_key.expose_secret())
        .args(["get", "SECRET"])
        .output()
        .unwrap();
    assert_failure(&output);
}

// --- Helpers ---

fn find_identity_key(t: &Test) -> String {
    let home = t.home.path();

    // Check global identity first
    let global_id = home.join(".dugout").join("identity");
    if global_id.exists() {
        return std::fs::read_to_string(global_id).unwrap();
    }

    // Search in keys directory
    let keys_dir = home.join(".dugout").join("keys");
    if keys_dir.exists() {
        for entry in std::fs::read_dir(&keys_dir).unwrap().flatten() {
            let key_file = entry.path().join("identity.key");
            if key_file.exists() {
                return std::fs::read_to_string(key_file).unwrap();
            }
        }
    }

    panic!("No identity key found in {:?}", home);
}

fn remove_all_identities(t: &Test) {
    let home = t.home.path();
    let keys_dir = home.join(".dugout").join("keys");
    if keys_dir.exists() {
        std::fs::remove_dir_all(&keys_dir).unwrap();
    }
    let global_id = home.join(".dugout").join("identity");
    if global_id.exists() {
        std::fs::remove_file(&global_id).unwrap();
    }
}
