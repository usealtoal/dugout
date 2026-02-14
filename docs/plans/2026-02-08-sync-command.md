# dugout sync — Design Plan

## Problem

After `git pull`, team membership changes, or key rotation by another member,
local secrets may not be encrypted for the current recipient set. Users need
a single command to bring everything into a consistent state.

## Design

### Core Behavior

`dugout sync` re-encrypts all secrets for the current `[recipients]` set.

It's idempotent — running it twice produces the same result. The operation
is always correct because it reads the current recipient list and re-encrypts
everything for exactly those keys.

### Recipient Fingerprint (sync detection)

To distinguish "already in sync" from "needed re-encryption," we store a
`recipients_hash` in the `[dugout]` config section. This is a sha256 hash
of the sorted, concatenated recipient public keys.

- On every write (set, import, rotate, sync, admit) → update the hash
- On sync → compare current recipients hash vs stored hash
- Match → "already in sync" (skip re-encryption)
- Mismatch → re-encrypt all, update hash

This makes sync O(1) to detect no-ops, but still does the full re-encrypt
when needed (which is fast anyway — ~100µs per secret).

### Vault API

Add to `Vault`:

```rust
/// Compute sha256 fingerprint of the current recipient set.
pub fn recipients_fingerprint(&self) -> String

/// Check if secrets are encrypted for the current recipient set.
pub fn needs_sync(&self) -> bool

/// Sync all secrets for the current recipient set.
/// Returns (secrets_count, recipients_count, was_needed).
pub fn sync(&mut self) -> Result<SyncResult>
```

New domain type:

```rust
pub struct SyncResult {
    pub secrets: usize,
    pub recipients: usize,
    pub was_needed: bool,
}
```

### CLI

```
dugout sync              # re-encrypt if needed
dugout sync --dry-run    # show if sync is needed without doing it
dugout sync --force      # re-encrypt even if fingerprint matches
```

Output examples:
- `synced (12 secrets, 3 recipients)`
- `already in sync`
- `would sync (12 secrets, 3 recipients)` (dry-run)

### Config Change

```toml
[dugout]
version = "0.1.6"
recipients_hash = "a1b2c3..."  # new field, optional
```

The hash field is optional for backward compat. If missing, sync
always re-encrypts (safe default). Future writes populate it.

### Fingerprint Update Points

Every operation that writes secrets updates the hash:
- `Vault::set()` 
- `Vault::reencrypt_all()` (called by add_recipient, remove_recipient, rotate)
- `Vault::import()`
- `Vault::sync()` (new)

This is done in a single helper: `Vault::update_recipients_hash()`.

### Error Handling

- Identity not in recipients → `AccessDenied` (existing error)
- Can't decrypt a secret → Report which key failed, continue with others, 
  return error at end with summary
- Empty vault (no secrets) → "already in sync" (nothing to do)

### Testing

1. **Unit tests (vault.rs):**
   - `recipients_fingerprint()` deterministic for same set
   - `recipients_fingerprint()` changes when recipients change
   - `needs_sync()` returns false after fresh init
   - `needs_sync()` returns true after manual recipient edit
   - `sync()` returns was_needed=false when already synced
   - `sync()` returns was_needed=true and re-encrypts after change

2. **CLI integration tests (tests/cli/):**
   - `dugout sync` on fresh vault → "already in sync"
   - `dugout sync` after adding recipient manually → syncs
   - `dugout sync --dry-run` → reports but doesn't change
   - `dugout sync --force` → always re-encrypts
   - `dugout sync` with no secrets → "already in sync"
   - `dugout sync` with corrupted secret → error with key name

3. **Workflow tests (tests/workflows.rs or tests/sync.rs):**
   - Full flow: init → set secrets → add recipient → sync → verify new recipient can decrypt
   - Full flow: init → set secrets → remove recipient → sync → verify re-encrypted
   - Fingerprint persists across open/close cycles
   - Multiple syncs are idempotent

## File Changes

### New files:
- `src/cli/sync.rs` — CLI command (flat, top-level like setup.rs)
- `src/core/domain/sync.rs` — SyncResult type
- `tests/sync.rs` — integration tests

### Modified files:
- `src/cli/mod.rs` — add Sync subcommand + flags
- `src/core/vault.rs` — add sync(), needs_sync(), recipients_fingerprint(), update_recipients_hash()
- `src/core/domain/mod.rs` — pub use sync module
- `src/core/config.rs` — add recipients_hash to Meta, skip_serializing_if None
- `Cargo.toml` — add sha2 crate (if not already present)

### Code reuse:
- `Vault::reencrypt_all()` — already exists, sync calls it
- `get_recipients_as_strings()` — already exists in vault.rs
- `CipherBackend::encrypt/decrypt` — unchanged
- Output helpers — existing output.rs functions

## Implementation Order

1. Add sha2 dep + recipients_hash to Config.Meta
2. Add domain::SyncResult
3. Add Vault methods: recipients_fingerprint, needs_sync, update_recipients_hash, sync
4. Wire update_recipients_hash into existing write paths (set, reencrypt_all, import)
5. Add CLI command (sync.rs + mod.rs wiring)
6. Write tests
7. Verify all existing tests still pass
