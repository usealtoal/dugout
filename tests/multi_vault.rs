//! Multi-vault integration tests.

mod support;

use support::Test;

#[test]
fn test_init_creates_default_vault() {
    let t = Test::new();
    let output = t.init_cmd("alice");
    assert!(output.status.success());
    assert!(t.dir.path().join(".dugout.toml").exists());
}

#[test]
fn test_init_creates_named_vault() {
    let t = Test::new();
    let output = t.init_vault("alice", "dev");
    assert!(output.status.success());
    assert!(t.dir.path().join(".dugout.dev.toml").exists());
    assert!(!t.dir.path().join(".dugout.toml").exists());
}

#[test]
fn test_vault_isolation() {
    let t = Test::new();

    // Create two vaults
    t.init_cmd("alice");
    t.init_vault("alice", "dev");

    // Set different secrets in each (must use -v flag with multiple vaults)
    t.set_vault("default", "PROD_SECRET", "prod_value");
    t.set_vault("dev", "DEV_SECRET", "dev_value");

    // Verify isolation
    let prod_get = t.get_vault("default", "PROD_SECRET");
    assert!(prod_get.status.success());
    assert!(String::from_utf8_lossy(&prod_get.stdout).contains("prod_value"));

    let dev_get = t.get_vault("dev", "DEV_SECRET");
    assert!(dev_get.status.success());
    assert!(String::from_utf8_lossy(&dev_get.stdout).contains("dev_value"));

    // Cross-vault access should fail
    let cross_get = t.get_vault("dev", "PROD_SECRET");
    assert!(!cross_get.status.success());
}

#[test]
fn test_vault_list_shows_all_vaults() {
    let t = Test::new();

    // Create multiple vaults
    t.init_cmd("alice");
    t.init_vault("alice", "dev");
    t.init_vault("alice", "prod");

    // Add some secrets
    t.set("SECRET", "value");
    t.set_vault("dev", "SECRET", "value");

    // List vaults
    let output = t.vault_list();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("default"));
    assert!(stdout.contains("dev"));
    assert!(stdout.contains("prod"));
}

#[test]
fn test_single_vault_no_flag_needed() {
    let t = Test::new();
    t.init_cmd("alice");

    // Should work without -v flag
    let set = t.set("KEY", "value");
    assert!(set.status.success());

    let get = t.get("KEY");
    assert!(get.status.success());
}

#[test]
fn test_multiple_vaults_requires_flag() {
    let t = Test::new();
    t.init_cmd("alice");
    t.init_vault("alice", "dev");

    // Without flag should fail with helpful error
    let get = t.cmd().args(["get", "KEY"]).output().unwrap();
    assert!(!get.status.success());

    let stderr = String::from_utf8_lossy(&get.stderr);
    assert!(stderr.contains("multiple vaults"));
    assert!(stderr.contains("--vault"));
}

#[test]
fn test_env_var_selects_vault() {
    let t = Test::new();
    t.init_cmd("alice");
    t.init_vault("alice", "dev");

    t.set("DEFAULT_KEY", "default_value");
    t.set_vault("dev", "DEV_KEY", "dev_value");

    // Use env var to select vault
    let get = t.cmd()
        .env("DUGOUT_VAULT", "dev")
        .args(["get", "DEV_KEY"])
        .output()
        .unwrap();

    assert!(get.status.success());
    assert!(String::from_utf8_lossy(&get.stdout).contains("dev_value"));
}

#[test]
fn test_env_var_isolates_vault() {
    let t = Test::new();
    t.init_cmd("alice");
    t.init_vault("alice", "dev");

    t.set_vault("default", "DEFAULT_KEY", "default_value");
    t.set_vault("dev", "DEV_KEY", "dev_value");

    // Env var selects dev vault, so DEFAULT_KEY (only in default) should not be found
    let get = t.cmd()
        .env("DUGOUT_VAULT", "dev")
        .args(["get", "DEFAULT_KEY"])
        .output()
        .unwrap();

    // Should fail because DEFAULT_KEY doesn't exist in dev vault
    assert!(!get.status.success());
}

#[test]
fn test_flag_overrides_env_var() {
    let t = Test::new();
    t.init_cmd("alice");
    t.init_vault("alice", "dev");

    t.set_vault("default", "DEFAULT_KEY", "default_value");
    t.set_vault("dev", "DEV_KEY", "dev_value");

    // Flag (--vault default) should override env var (DUGOUT_VAULT=dev)
    let get = t.cmd()
        .env("DUGOUT_VAULT", "dev")
        .args(["--vault", "default", "get", "DEFAULT_KEY"])
        .output()
        .unwrap();

    // Should succeed because flag overrides env var to select default vault
    assert!(get.status.success());
    assert!(String::from_utf8_lossy(&get.stdout).contains("default_value"));
}

#[test]
fn test_dot_uses_default_vault() {
    // dugout . should always use .dugout.toml even with multiple vaults
    let t = Test::new();
    t.init_cmd("alice");
    t.init_vault("alice", "dev");

    // Create a package.json so dot command has something to detect
    std::fs::write(t.dir.path().join("package.json"), r#"{"scripts":{"dev":"echo ok"}}"#).unwrap();

    t.set("SECRET", "value");

    // dugout . should work without requiring -v
    // (actual execution depends on npm being available, just test it doesn't error about multiple vaults)
    let dot = t.cmd().arg(".").output().unwrap();
    let stderr = String::from_utf8_lossy(&dot.stderr);
    assert!(!stderr.contains("multiple vaults"));
}

#[test]
fn test_legacy_request_migration() {
    let t = Test::new();
    t.init_cmd("alice");

    // Create a legacy request file in old location
    let legacy_dir = t.dir.path().join(".dugout/requests");
    std::fs::create_dir_all(&legacy_dir).unwrap();
    let legacy_file = legacy_dir.join("bob.pub");
    std::fs::write(&legacy_file, "age1testpubkey123").unwrap();

    // Verify legacy file exists
    assert!(legacy_file.exists());

    // Run pending command - this triggers migration
    let output = t.cmd().args(["pending"]).output().unwrap();
    assert!(output.status.success());

    // Verify file was migrated to new location
    let new_file = t.dir.path().join(".dugout/requests/default/bob.pub");
    assert!(new_file.exists(), "Request file should be migrated to new location");

    // Verify old file was removed
    assert!(!legacy_file.exists(), "Legacy request file should be removed after migration");
}
