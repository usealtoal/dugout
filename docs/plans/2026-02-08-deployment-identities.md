# Deployment Identities Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable CI runners and production servers to decrypt dugout secrets via environment variable identity injection, with zero cloud dependencies.

**Architecture:** Extend identity resolution to check `DUGOUT_IDENTITY` (inline key) and `DUGOUT_IDENTITY_FILE` (path) env vars before filesystem lookup. Enhance `dugout setup` with `--name` and `--output` flags for non-interactive CI bootstrapping. Add deployment documentation with CI workflow examples.

**Tech Stack:** Rust, clap (CLI), age (crypto), tempfile (tests)

---

### Task 1: DUGOUT_IDENTITY env var support in Identity

**Files:**
- Modify: `src/core/domain/identity.rs` — add `from_env()` and `from_env_file()` methods
- Test: `tests/identity.rs` — add env var resolution tests

**Step 1: Write failing tests**

Add to `tests/identity.rs`:

```rust
#[test]
fn test_identity_from_env_var() {
    use dugout::Vault;
    use std::env;
    use tempfile::TempDir;

    let original_dir = env::current_dir().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let home_dir = TempDir::new().unwrap();

    env::set_var("HOME", home_dir.path());
    env::set_current_dir(&temp_dir).unwrap();

    // Generate a throwaway identity to get a valid key string
    let identity = age::x25519::Identity::generate();
    let pubkey = identity.to_public().to_string();
    use age::secrecy::ExposeSecret;
    let secret_key = identity.to_string();
    let secret_key_str = secret_key.expose_secret().to_string();

    // Init vault with this identity's public key as recipient
    let vault = Vault::init("ci-user", None, None, None).unwrap();
    drop(vault);

    // Remove the filesystem identity so env var is the only path
    let keys_dir = home_dir.path().join(".dugout").join("keys");
    if keys_dir.exists() {
        std::fs::remove_dir_all(&keys_dir).unwrap();
    }
    let global_id = home_dir.path().join(".dugout").join("identity");
    if global_id.exists() {
        std::fs::remove_file(&global_id).unwrap();
    }

    // Set the env var
    env::set_var("DUGOUT_IDENTITY", &secret_key_str);

    // Should be able to open vault via env var identity
    // (only if the pubkey matches a recipient — we need to add it)
    // For now just test that Identity::from_env() works
    let loaded = dugout::core::domain::Identity::from_env();
    assert!(loaded.is_some(), "Should load identity from DUGOUT_IDENTITY env var");

    let loaded = loaded.unwrap();
    assert_eq!(loaded.public_key(), pubkey);

    env::remove_var("DUGOUT_IDENTITY");
    let _ = env::set_current_dir(&original_dir);
}

#[test]
fn test_identity_from_env_file() {
    use std::env;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();

    let identity = age::x25519::Identity::generate();
    let pubkey = identity.to_public().to_string();
    use age::secrecy::ExposeSecret;
    let secret_key = identity.to_string();

    // Write key to a file
    let key_file = temp_dir.path().join("ci-identity.key");
    std::fs::write(&key_file, secret_key.expose_secret()).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&key_file, std::fs::Permissions::from_mode(0o600)).unwrap();
    }

    env::set_var("DUGOUT_IDENTITY_FILE", key_file.to_str().unwrap());

    let loaded = dugout::core::domain::Identity::from_env();
    assert!(loaded.is_some(), "Should load identity from DUGOUT_IDENTITY_FILE");
    assert_eq!(loaded.unwrap().public_key(), pubkey);

    env::remove_var("DUGOUT_IDENTITY_FILE");
}

#[test]
fn test_identity_env_var_takes_precedence() {
    use std::env;
    use tempfile::TempDir;

    let home_dir = TempDir::new().unwrap();

    // Create a global identity on disk
    env::set_var("HOME", home_dir.path());
    let disk_identity = dugout::core::domain::Identity::generate_global().unwrap();
    let disk_pubkey = disk_identity.public_key();
    drop(disk_identity);

    // Create a different identity for env var
    let env_identity = age::x25519::Identity::generate();
    let env_pubkey = env_identity.to_public().to_string();
    use age::secrecy::ExposeSecret;
    let secret_key = env_identity.to_string();
    env::set_var("DUGOUT_IDENTITY", secret_key.expose_secret());

    // Env should win over disk
    let loaded = dugout::core::domain::Identity::from_env();
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().public_key(), env_pubkey);
    assert_ne!(env_pubkey, disk_pubkey, "Keys should be different");

    env::remove_var("DUGOUT_IDENTITY");
}

#[test]
fn test_identity_invalid_env_var_ignored() {
    use std::env;

    env::set_var("DUGOUT_IDENTITY", "not-a-valid-age-key");

    let loaded = dugout::core::domain::Identity::from_env();
    assert!(loaded.is_none(), "Invalid key should return None, not panic");

    env::remove_var("DUGOUT_IDENTITY");
}
```

**Step 2: Run tests to verify they fail**

Run: `. "$HOME/.cargo/env" && cargo test --test identity -- --test-threads=1 2>&1 | grep "FAILED\|error"`
Expected: compilation errors — `from_env` doesn't exist yet

**Step 3: Implement Identity::from_env()**

Add to `src/core/domain/identity.rs`, after the existing `impl Identity` block's methods, before the closing `}`:

```rust
    /// Load identity from environment variables.
    ///
    /// Checks in order:
    /// 1. `DUGOUT_IDENTITY` — raw AGE-SECRET-KEY inline
    /// 2. `DUGOUT_IDENTITY_FILE` — path to a file containing the key
    ///
    /// Returns `None` if neither is set or the key is invalid.
    pub fn from_env() -> Option<Self> {
        // 1. Inline key
        if let Ok(key) = std::env::var("DUGOUT_IDENTITY") {
            debug!("found DUGOUT_IDENTITY env var");
            if let Ok(inner) = key.trim().parse::<x25519::Identity>() {
                return Some(Self {
                    inner,
                    path: PathBuf::from("<env:DUGOUT_IDENTITY>"),
                });
            }
            debug!("DUGOUT_IDENTITY value is not a valid age key");
        }

        // 2. Key file path
        if let Ok(path_str) = std::env::var("DUGOUT_IDENTITY_FILE") {
            debug!(path = %path_str, "found DUGOUT_IDENTITY_FILE env var");
            let path = PathBuf::from(&path_str);
            if !path.exists() {
                debug!("DUGOUT_IDENTITY_FILE path does not exist");
                return None;
            }

            // Verify permissions on Unix
            #[cfg(unix)]
            if Self::validate_file_permissions(&path, 0o600).is_err() {
                debug!("DUGOUT_IDENTITY_FILE has insecure permissions");
                return None;
            }

            if let Ok(contents) = fs::read_to_string(&path) {
                if let Ok(inner) = contents.trim().parse::<x25519::Identity>() {
                    return Some(Self { inner, path });
                }
            }
            debug!("DUGOUT_IDENTITY_FILE contents are not a valid age key");
        }

        None
    }
```

**Step 4: Export from lib.rs if needed**

Check that `dugout::core::domain::Identity` is accessible from tests. If not, ensure `pub use` chain is correct in `src/lib.rs` and `src/core/mod.rs` and `src/core/domain/mod.rs`.

Run: `. "$HOME/.cargo/env" && cargo test --test identity -- --test-threads=1 -v 2>&1 | tail -20`

**Step 5: Run tests to verify they pass**

Run: `. "$HOME/.cargo/env" && cargo test --test identity -- --test-threads=1 -v`
Expected: all identity tests PASS

**Step 6: Commit**

```bash
git add src/core/domain/identity.rs tests/identity.rs
git commit -m "feat: DUGOUT_IDENTITY env var support for CI/CD"
```

---

### Task 2: Wire env identity into Vault::open()

**Files:**
- Modify: `src/core/vault.rs` — check `Identity::from_env()` before filesystem lookup
- Test: `tests/deploy.rs` — new file: end-to-end deployment scenario tests

**Step 1: Write failing test**

Create `tests/deploy.rs`:

```rust
//! Deployment and CI/CD scenario tests.

mod support;
use support::*;

#[test]
fn test_decrypt_via_env_identity() {
    let t = Test::init("alice");

    // Set a secret
    let output = t.set("API_KEY", "sk_live_abc123");
    assert_success(&output);

    // Get the identity key content
    let home_dir = t.home.path();
    let global_identity_path = home_dir.join(".dugout").join("identity");
    let identity_content = std::fs::read_to_string(&global_identity_path).unwrap();

    // Remove all identity files to force env-only auth
    let keys_dir = home_dir.join(".dugout").join("keys");
    if keys_dir.exists() {
        std::fs::remove_dir_all(&keys_dir).unwrap();
    }
    std::fs::remove_file(&global_identity_path).unwrap();

    // Set env var and try to decrypt
    let output = t.cmd()
        .env("DUGOUT_IDENTITY", identity_content.trim())
        .args(["get", "API_KEY"])
        .output()
        .unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "sk_live_abc123");
}

#[test]
fn test_decrypt_via_env_identity_file() {
    let t = Test::init("alice");

    let output = t.set("DB_URL", "postgres://localhost/prod");
    assert_success(&output);

    // Copy identity to a separate file
    let home_dir = t.home.path();
    let global_identity_path = home_dir.join(".dugout").join("identity");
    let identity_content = std::fs::read_to_string(&global_identity_path).unwrap();

    let ci_key_path = t.dir.path().join("ci-identity.key");
    std::fs::write(&ci_key_path, &identity_content).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&ci_key_path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }

    // Remove all identity files
    let keys_dir = home_dir.join(".dugout").join("keys");
    if keys_dir.exists() {
        std::fs::remove_dir_all(&keys_dir).unwrap();
    }
    std::fs::remove_file(&global_identity_path).unwrap();

    // Set env var and try to decrypt
    let output = t.cmd()
        .env("DUGOUT_IDENTITY_FILE", ci_key_path.to_str().unwrap())
        .args(["get", "DB_URL"])
        .output()
        .unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "postgres://localhost/prod");
}

#[test]
fn test_run_via_env_identity() {
    let t = Test::init("alice");

    t.set("MY_SECRET", "hunter2");

    let home_dir = t.home.path();
    let global_identity_path = home_dir.join(".dugout").join("identity");
    let identity_content = std::fs::read_to_string(&global_identity_path).unwrap();

    // Remove filesystem identities
    let keys_dir = home_dir.join(".dugout").join("keys");
    if keys_dir.exists() {
        std::fs::remove_dir_all(&keys_dir).unwrap();
    }
    std::fs::remove_file(&global_identity_path).unwrap();

    // Run a command that prints the env var
    let output = t.cmd()
        .env("DUGOUT_IDENTITY", identity_content.trim())
        .args(["run", "--", "printenv", "MY_SECRET"])
        .output()
        .unwrap();
    assert_success(&output);
    assert_stdout_contains(&output, "hunter2");
}

#[test]
fn test_no_identity_gives_helpful_error() {
    let t = Test::init("alice");
    t.set("SECRET", "value");

    // Remove ALL identity sources
    let home_dir = t.home.path();
    let keys_dir = home_dir.join(".dugout").join("keys");
    if keys_dir.exists() {
        std::fs::remove_dir_all(&keys_dir).unwrap();
    }
    let global_identity_path = home_dir.join(".dugout").join("identity");
    if global_identity_path.exists() {
        std::fs::remove_file(&global_identity_path).unwrap();
    }

    let output = t.cmd()
        .args(["get", "SECRET"])
        .output()
        .unwrap();
    assert_failure(&output);
    let err = stderr(&output);
    // Should mention DUGOUT_IDENTITY or setup
    assert!(
        err.contains("DUGOUT_IDENTITY") || err.contains("setup") || err.contains("identity"),
        "Error should guide user to set up identity, got: {err}"
    );
}
```

**Step 2: Run to verify failure**

Run: `. "$HOME/.cargo/env" && cargo test --test deploy -- --test-threads=1 -v 2>&1 | tail -20`
Expected: `test_decrypt_via_env_identity` FAILS (env var not checked in Vault::open)

**Step 3: Modify Vault::open() to check env identity**

In `src/core/vault.rs`, modify the `open()` method. Replace the identity resolution block with:

```rust
    pub fn open() -> Result<Self> {
        let config = Config::load()?;
        let project_id = config.project_id();

        // Identity resolution order:
        // 1. DUGOUT_IDENTITY / DUGOUT_IDENTITY_FILE env vars (CI/CD)
        // 2. Project-local identity (~/.dugout/keys/<project>/)
        // 3. Global identity (~/.dugout/identity)
        let identity = Identity::from_env()
            .filter(|id| identity_has_access(&config, id))
            .or_else(|| {
                store::load_identity(&project_id)
                    .ok()
                    .filter(|id| identity_has_access(&config, id))
            })
            .or_else(|| {
                Identity::has_global()
                    .ok()
                    .filter(|has| *has)
                    .and_then(|_| Identity::load_global().ok())
                    .filter(|id| identity_has_access(&config, id))
            })
            .ok_or(ConfigError::AccessDenied)?;

        let backend = cipher::CipherBackend::from_config(&config)?;

        Ok(Self {
            config,
            project_id,
            identity,
            backend,
        })
    }
```

**Step 4: Run tests**

Run: `. "$HOME/.cargo/env" && cargo test --test deploy -- --test-threads=1 -v`
Expected: all deploy tests PASS

Run: `. "$HOME/.cargo/env" && cargo test -- --test-threads=1 2>&1 | grep "test result"`
Expected: all tests PASS (no regressions)

**Step 5: Commit**

```bash
git add src/core/vault.rs tests/deploy.rs
git commit -m "feat: env var identity resolution in Vault::open()"
```

---

### Task 3: Enhance dugout setup with --name and --output

**Files:**
- Modify: `src/cli/mod.rs` — add `--name` and `--output` args to Setup variant
- Modify: `src/cli/setup.rs` — implement new flags
- Test: `tests/deploy.rs` — add setup output tests

**Step 1: Write failing tests**

Add to `tests/deploy.rs`:

```rust
#[test]
fn test_setup_with_name_flag() {
    let t = Test::new();

    let output = t.cmd()
        .args(["setup", "--name", "ci-runner"])
        .output()
        .unwrap();
    assert_success(&output);

    // Should create identity
    let output = t.cmd()
        .arg("whoami")
        .output()
        .unwrap();
    assert_success(&output);
    let pubkey = stdout(&output);
    assert!(pubkey.trim().starts_with("age1"));
}

#[test]
fn test_setup_output_to_stdout() {
    let t = Test::new();

    let output = t.cmd()
        .args(["setup", "--output", "-"])
        .output()
        .unwrap();
    assert_success(&output);

    let out = stdout(&output);
    assert!(
        out.contains("AGE-SECRET-KEY-"),
        "Should print private key to stdout, got: {out}"
    );
}

#[test]
fn test_setup_output_to_file() {
    let t = Test::new();

    let key_path = t.dir.path().join("ci.key");
    let output = t.cmd()
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
```

**Step 2: Run to verify failure**

Run: `. "$HOME/.cargo/env" && cargo test --test deploy test_setup -- --test-threads=1 -v`
Expected: FAIL — `--name` and `--output` flags not recognized

**Step 3: Add CLI flags**

In `src/cli/mod.rs`, update the `Setup` variant:

```rust
    /// Generate global identity at ~/.dugout/identity
    Setup {
        /// Overwrite existing identity
        #[arg(short, long)]
        force: bool,
        /// Identity name (skips interactive prompt)
        #[arg(short, long)]
        name: Option<String>,
        /// Write private key to path (use - for stdout)
        #[arg(short, long, value_name = "PATH")]
        output: Option<String>,
    },
```

Update the match arm in `execute()`:

```rust
        Setup { force, name, output } => setup::execute(force, name, output),
```

**Step 4: Implement in setup.rs**

Replace `src/cli/setup.rs`:

```rust
//! Setup command - generate global identity.

use crate::cli::output;
use crate::core::domain::Identity;
use crate::error::Result;

/// Generate global identity.
pub fn execute(force: bool, name: Option<String>, output_path: Option<String>) -> Result<()> {
    // Check if identity already exists
    if Identity::has_global()? && !force {
        let pubkey = Identity::load_global_pubkey()?;
        output::warn("identity already exists");
        output::hint(&format!("public key: {}", pubkey));
        output::hint("use --force to overwrite");
        return Ok(());
    }

    let identity = Identity::generate_global()?;
    let pubkey = identity.public_key();

    // Output private key if requested
    if let Some(ref path) = output_path {
        use age::secrecy::ExposeSecret;
        let secret = identity.as_age().to_string();
        let key_str = secret.expose_secret();

        if path == "-" {
            // Print to stdout (raw, for piping)
            output::raw(key_str);
        } else {
            // Write to file
            std::fs::write(path, format!("{}\n", key_str))
                .map_err(|e| crate::error::StoreError::WriteFailed(e))?;

            // Set restrictive permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
                    .map_err(|e| crate::error::StoreError::WriteFailed(e))?;
            }

            output::success(&format!("private key written to {}", path));
        }
    }

    // Always print public key to stderr (so stdout is clean for piping)
    if output_path.as_deref() == Some("-") {
        eprintln!("public key: {}", pubkey);
    } else {
        output::success("generated identity");
        output::hint(&format!("public key: {}", pubkey));
    }

    Ok(())
}
```

**Step 5: Run tests**

Run: `. "$HOME/.cargo/env" && cargo test --test deploy -- --test-threads=1 -v`
Expected: all setup tests PASS

Run: `. "$HOME/.cargo/env" && cargo test -- --test-threads=1 2>&1 | grep "test result"`
Expected: all tests PASS

**Step 6: Commit**

```bash
git add src/cli/mod.rs src/cli/setup.rs tests/deploy.rs
git commit -m "feat: setup --name and --output flags for CI bootstrapping"
```

---

### Task 4: Improve error message when no identity is found

**Files:**
- Modify: `src/core/vault.rs` — better error for AccessDenied when no identity exists
- Modify: `src/error.rs` — add `NoIdentity` error variant (or improve AccessDenied message)

**Step 1: Check current error types**

Read `src/error.rs` to see what error variants exist and how `AccessDenied` is defined.

**Step 2: Update the error message**

The `AccessDenied` error should include guidance. Either:
- Add a `NoIdentity` variant with a message suggesting `DUGOUT_IDENTITY` or `dugout setup`
- Or make the `Display` impl for `AccessDenied` include this guidance

The test in Task 2 (`test_no_identity_gives_helpful_error`) validates this.

**Step 3: Run all tests**

Run: `. "$HOME/.cargo/env" && cargo test -- --test-threads=1 2>&1 | grep "test result"`
Expected: all PASS

**Step 4: Commit**

```bash
git add src/error.rs src/core/vault.rs
git commit -m "feat: helpful error when no identity found"
```

---

### Task 5: Deployment documentation

**Files:**
- Create: `docs/deploy.md` — main deployment guide
- Modify: `README.md` — add Deployment section linking to docs

**Step 1: Write docs/deploy.md**

```markdown
# Deployment Guide

dugout works in CI/CD and production with zero cloud dependencies.

## Quick Start

1. Generate a CI identity:
   ```bash
   dugout setup --output ci.key
   ```

2. Store the private key as a CI secret (e.g., `DUGOUT_IDENTITY` in GitHub Actions)

3. Admit the identity to your project:
   ```bash
   dugout team add ci "$(dugout whoami)"
   ```

4. In CI, decrypt and run:
   ```yaml
   - env:
       DUGOUT_IDENTITY: ${{ secrets.DUGOUT_IDENTITY }}
     run: dugout run -- ./deploy.sh
   ```

## Identity Resolution

dugout checks for identities in this order:

| Priority | Source | Use case |
|---|---|---|
| 1 | `DUGOUT_IDENTITY` env var | CI/CD (inline key) |
| 2 | `DUGOUT_IDENTITY_FILE` env var | Servers (key file path) |
| 3 | `.dugout/keys/<project>/identity.key` | Developer (project-local) |
| 4 | `~/.dugout/identity` | Developer (global) |

## CI/CD Examples

### GitHub Actions

```yaml
name: Deploy
on: push
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install dugout
        run: curl -LsSf https://raw.githubusercontent.com/usealtoal/dugout/main/scripts/install.sh | sh
      - name: Deploy
        env:
          DUGOUT_IDENTITY: ${{ secrets.DUGOUT_IDENTITY }}
        run: dugout run -- ./deploy.sh
```

### GitLab CI

```yaml
deploy:
  script:
    - curl -LsSf https://raw.githubusercontent.com/usealtoal/dugout/main/scripts/install.sh | sh
    - dugout run -- ./deploy.sh
  variables:
    DUGOUT_IDENTITY: $DUGOUT_IDENTITY
```

### Docker

```dockerfile
FROM debian:bookworm-slim
RUN curl -LsSf https://raw.githubusercontent.com/usealtoal/dugout/main/scripts/install.sh | sh
COPY . /app
WORKDIR /app
CMD ["dugout", "run", "--", "./start.sh"]
```

```bash
docker run -e DUGOUT_IDENTITY="$KEY" myapp
```

## Security Notes

- Never bake `DUGOUT_IDENTITY` into a Docker image
- Use your CI provider's secret storage for the key
- The identity is a single age private key (~200 bytes)
- Rotate by generating a new identity, admitting it, and removing the old one
```

**Step 2: Add README section**

Add a brief "Deployment" section to `README.md` after the Team Workflow section, linking to the full guide:

```markdown
## Deployment

dugout works in CI/CD and production. Set `DUGOUT_IDENTITY` and go:

```yaml
# GitHub Actions
- env:
    DUGOUT_IDENTITY: ${{ secrets.DUGOUT_IDENTITY }}
  run: dugout run -- ./deploy.sh
```

See the full [Deployment Guide](docs/deploy.md) for GitLab, Docker, and more.
```

**Step 3: Commit**

```bash
git add docs/deploy.md README.md
git commit -m "docs: deployment guide with CI/CD examples"
```

---

### Task 6: Clippy, fmt, full test suite

**Step 1: Format**

Run: `. "$HOME/.cargo/env" && cargo fmt`

**Step 2: Clippy**

Run: `. "$HOME/.cargo/env" && cargo clippy -- -D warnings`
Fix any warnings.

**Step 3: Full test suite**

Run: `. "$HOME/.cargo/env" && cargo test -- --test-threads=1`
Expected: all tests PASS (270+ existing + ~10 new)

**Step 4: Final commit if needed**

```bash
git add -A
git commit -m "chore: fmt + clippy clean"
```

---

### Task 7: Create PR

**Step 1: Push branch**

```bash
git push origin feat/deployment-identities
```

**Step 2: Create PR**

```bash
gh pr create \
  --title "feat: deployment identities (DUGOUT_IDENTITY env var)" \
  --body "## What

Enables CI runners and production servers to decrypt dugout secrets via environment variable identity injection.

## Changes

- \`DUGOUT_IDENTITY\` env var: set to an age private key, dugout uses it for decryption
- \`DUGOUT_IDENTITY_FILE\` env var: path to a key file
- \`dugout setup --name ci-prod --output ci.key\`: non-interactive identity generation
- Identity resolution order: env var → env file → project key → global key
- Deployment guide with GitHub Actions, GitLab CI, Docker examples
- ~10 new integration tests

## Testing

\`\`\`bash
cargo test -- --test-threads=1
\`\`\`

All existing tests pass. New tests in \`tests/deploy.rs\` and \`tests/identity.rs\`." \
  --base main
```
