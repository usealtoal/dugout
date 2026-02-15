# Multi-Vault Support — Design Plan

## Problem

Teams often need different secrets for different environments (dev, staging, prod)
with different access control. A developer might have access to dev secrets but
not prod. Currently, dugout supports only a single `.dugout.toml` vault per repo,
forcing teams to use workarounds like separate branches or external tooling.

## Design

### Core Concept

Support multiple vault files in a single repository, each with independent
secrets and recipients. Selection via CLI flag or environment variable.

### Vault File Naming

```
.dugout.toml          # default vault (no suffix)
.dugout.dev.toml      # dev vault
.dugout.staging.toml  # staging vault
.dugout.prod.toml     # prod vault
```

Pattern: `.dugout.{name}.toml` where `{name}` is the vault identifier.
The default vault remains `.dugout.toml` for backward compatibility.

### Vault Selection

**Flag:** `-v` / `--vault`
**Environment variable:** `DUGOUT_VAULT`
**Precedence:** flag > env var > default

```bash
# Using flag
dugout -v prod get DATABASE_URL
dugout --vault staging set API_KEY "secret"

# Using environment variable
export DUGOUT_VAULT=prod
dugout get DATABASE_URL

# Flag overrides env var
DUGOUT_VAULT=dev dugout -v prod get DATABASE_URL  # uses prod
```

### Default Behavior

**Single vault exists:** Commands use it automatically (no flag needed).

**Multiple vaults exist:**
- Commands that modify or read secrets require explicit `-v` flag
- Missing flag → error with guidance: `Multiple vaults found. Specify with -v: dugout -v prod <command>`
- Exception: `dugout .` always defaults to `.dugout.toml` (convenience for local dev)

This preserves backward compatibility — single-vault repos work exactly as before.

### Vault Independence

Each vault is fully independent:
- Own `[recipients]` section (different access control per vault)
- Own `[secrets]` section
- Own `recipients_hash` for sync detection
- No inheritance or sharing between vaults

This matches the security model: dev team ≠ prod team.

### Directory Structure

```
.dugout/
  identity              # shared — you're still you
  identity.pub
  requests/
    dev/                # per-vault request directories
      alice.pub
    prod/
      bob.pub
```

Identity is shared (one keypair per developer). Access requests are per-vault
because you request access to a specific vault.

### New Command: `dugout vault list`

Lists all vaults in the repository with status:

```bash
$ dugout vault list
VAULT      SECRETS  RECIPIENTS  ACCESS
default    12       3           ✓
dev        8        5           ✓
staging    8        3           ✓
prod       15       2           ✗
```

- Shows vault name, secret count, recipient count
- Shows access status (✓ if your key is a recipient, ✗ if not)
- Useful for discoverability and debugging access issues

### Modified Commands

**`dugout init`**
```bash
dugout init              # creates .dugout.toml
dugout init -v dev       # creates .dugout.dev.toml
```

**`dugout knock`**
```bash
# Single vault: just works
dugout knock

# Multiple vaults: requires explicit vault
dugout -v prod knock     # creates .dugout/requests/prod/<name>.pub
```

**`dugout admit`**
```bash
dugout -v prod admit alice   # reads from .dugout/requests/prod/alice.pub
```

**`dugout pending`**
```bash
dugout -v prod pending   # lists .dugout/requests/prod/*.pub
```

**All other commands** (`get`, `set`, `rm`, `list`, `run`, `env`, `sync`, `team *`, `secrets *`, `check *`)
follow the same pattern: `-v` flag selects vault, defaults apply as described above.

### CLI Structure

Add global flag to top-level Args:

```rust
#[derive(Parser)]
pub struct Args {
    /// Select vault (e.g., "dev", "prod"). Defaults to .dugout.toml
    #[arg(short = 'v', long = "vault", global = true, env = "DUGOUT_VAULT")]
    pub vault: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}
```

### Vault API Changes

```rust
impl Vault {
    /// Open vault with explicit name (None = default)
    pub fn open(vault: Option<&str>) -> Result<Self>

    /// Initialize new vault with explicit name (None = default)
    pub fn init(vault: Option<&str>, name: &str, kms: Option<&str>) -> Result<Self>

    /// List all vaults in current directory
    pub fn list_vaults() -> Result<Vec<VaultInfo>>

    /// Get vault file path for given name
    pub fn vault_path(vault: Option<&str>) -> PathBuf

    /// Get request directory for given vault
    pub fn request_dir(vault: Option<&str>) -> PathBuf
}

pub struct VaultInfo {
    pub name: String,           // "default", "dev", "prod"
    pub path: PathBuf,          // .dugout.toml, .dugout.dev.toml
    pub secret_count: usize,
    pub recipient_count: usize,
    pub has_access: bool,       // is current identity a recipient?
}
```

### Config Changes

No changes to `.dugout.toml` structure. Each vault file has the same format:

```toml
[dugout]
version = "0.1.2"
recipients_hash = "..."

[recipients]
alice = "age1..."

[secrets]
DATABASE_URL = "-----BEGIN AGE ENCRYPTED FILE-----..."
```

### Path Resolution

```rust
fn vault_path(vault: Option<&str>) -> PathBuf {
    match vault {
        None => PathBuf::from(".dugout.toml"),
        Some(name) => PathBuf::from(format!(".dugout.{}.toml", name)),
    }
}

fn request_dir(vault: Option<&str>) -> PathBuf {
    let base = PathBuf::from(".dugout/requests");
    match vault {
        None => base.join("default"),
        Some(name) => base.join(name),
    }
}
```

### Error Messages

When multiple vaults exist and no `-v` flag:

```
error: multiple vaults found

  .dugout.toml (default)
  .dugout.dev.toml
  .dugout.prod.toml

specify which vault to use:

  dugout -v dev get DATABASE_URL
  dugout --vault prod set API_KEY "..."

or set DUGOUT_VAULT environment variable:

  export DUGOUT_VAULT=dev
```

### Testing

1. **Unit tests (vault.rs):**
   - `vault_path(None)` returns `.dugout.toml`
   - `vault_path(Some("dev"))` returns `.dugout.dev.toml`
   - `request_dir(None)` returns `.dugout/requests/default`
   - `request_dir(Some("prod"))` returns `.dugout/requests/prod`
   - `list_vaults()` finds all `.dugout*.toml` files
   - `list_vaults()` correctly reports access status

2. **CLI integration tests:**
   - `dugout init` creates `.dugout.toml`
   - `dugout init -v dev` creates `.dugout.dev.toml`
   - `dugout -v dev set KEY val` writes to dev vault only
   - `dugout -v dev get KEY` reads from dev vault only
   - Single vault: commands work without `-v`
   - Multiple vaults: commands fail without `-v` (except `dugout .`)
   - `dugout .` uses default vault even with multiple vaults
   - `DUGOUT_VAULT=dev dugout get KEY` uses dev vault
   - Flag overrides env var
   - `dugout vault list` shows all vaults with correct info

3. **Workflow tests:**
   - Init default → init dev → set different secrets → verify isolation
   - Knock on prod → admit on prod → verify request dir structure
   - Add recipient to dev only → verify no access to prod
   - Sync on one vault doesn't affect others

## File Changes

### New files:
- `src/cli/vault/mod.rs` — vault subcommand (list)
- `src/cli/vault/list.rs` — vault list implementation
- `src/core/domain/vault_info.rs` — VaultInfo type
- `tests/multi_vault.rs` — integration tests

### Modified files:
- `src/cli/mod.rs` — add global `-v` flag, add `Vault(VaultCommand)` subcommand
- `src/core/vault.rs` — vault parameter on open/init, add list_vaults(), vault_path(), request_dir()
- `src/core/config.rs` — accept path parameter in load/save
- `src/core/constants.rs` — add request subdir constants
- `src/core/domain/mod.rs` — pub use vault_info
- `src/cli/knock.rs` — use vault-specific request dir
- `src/cli/admit.rs` — use vault-specific request dir
- `src/cli/pending.rs` — use vault-specific request dir
- `src/cli/init.rs` — pass vault to Vault::init
- All other CLI commands — pass vault from Args to Vault::open

### Backward Compatibility:
- Single-vault repos work exactly as before
- Existing `.dugout.toml` files unchanged
- No migration needed

## Implementation Order

1. Add VaultInfo domain type
2. Add vault_path() and request_dir() helpers to constants/vault
3. Modify Config::load/save to accept explicit path
4. Modify Vault::open/init to accept vault parameter
5. Add list_vaults() to Vault
6. Add global `-v` flag to CLI Args
7. Add multi-vault detection and error handling
8. Wire vault parameter through all CLI commands
9. Update knock/admit/pending for per-vault request dirs
10. Add `dugout vault list` command
11. Write tests
12. Verify all existing tests still pass
