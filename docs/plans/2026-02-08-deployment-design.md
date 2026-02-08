# Deployment & CI/CD Integration Design

> dugout secrets in production — cloud-agnostic, zero-config, backend-independent.

## Problem

dugout works great for dev teams. But production servers and CI runners need to decrypt secrets too. Today there's no documented or streamlined path for non-human identities.

## Design Principles

1. **Cloud-agnostic first** — the default path requires zero cloud dependencies
2. **Consistent UX** — after setup, every command is identical regardless of auth method
3. **Auto-detect auth** — dugout figures out how to decrypt without flags or config
4. **Progressive complexity** — simple path works in 30 seconds, advanced path unlocks cloud-native features

## Auth Resolution Order

When dugout needs to decrypt, it tries these in order:

```
1. DUGOUT_IDENTITY env var       → raw age private key inline
2. DUGOUT_IDENTITY_FILE env var  → path to age key file
3. .dugout/identity              → project-local identity
4. ~/.dugout/identity            → global identity
5. KMS (if configured)           → cloud provider decrypt API
6. Error                         → helpful message with setup instructions
```

This means:
- Developers hit steps 3-4 (their local key)
- CI runners hit step 1-2 (injected key)
- Production servers hit step 1-2 (injected key) or step 5 (KMS)
- Nobody ever specifies a flag — it just works

## Part 1: Service Identities (Cloud-Agnostic)

### Changes

**`dugout setup` enhancements:**

```
dugout setup                          # interactive (existing)
dugout setup --name ci-prod           # non-interactive, named identity
dugout setup --name ci-prod -o -      # print private key to stdout
dugout setup --name ci-prod -o key.txt  # write to file
```

New flags:
- `--name <name>` — skip interactive prompt, set identity name
- `-o, --output <path>` — write private key to path (`-` for stdout)

These compose for CI bootstrap:
```bash
# Generate a CI identity, capture the private key, print the public key
PRIVATE_KEY=$(dugout setup --name ci-prod -o -)
echo "Public key: $(dugout whoami)"
echo "Store this as DUGOUT_IDENTITY in your CI secrets: $PRIVATE_KEY"
```

**`DUGOUT_IDENTITY` env var support:**

In `Identity::resolve()` (or equivalent), check env before filesystem:

```rust
fn resolve_identity() -> Result<age::x25519::Identity> {
    // 1. Inline key from env
    if let Ok(key) = env::var("DUGOUT_IDENTITY") {
        return parse_age_identity(&key);
    }
    
    // 2. Key file path from env
    if let Ok(path) = env::var("DUGOUT_IDENTITY_FILE") {
        return load_identity_file(&path);
    }
    
    // 3-4. Project-local or global identity (existing logic)
    load_identity_from_filesystem()
}
```

**`dugout admit` for service identities:**

No changes needed — service identities are just age keys. The existing flow works:

```bash
# On your machine:
dugout admit ci-prod    # from knock request
# or
dugout team add ci-prod age1...  # direct add with public key
```

### CI/CD Workflow Examples

**GitHub Actions:**
```yaml
- name: Deploy with secrets
  env:
    DUGOUT_IDENTITY: ${{ secrets.DUGOUT_IDENTITY }}
  run: dugout run -- ./deploy.sh
```

**GitLab CI:**
```yaml
deploy:
  variables:
    DUGOUT_IDENTITY: $DUGOUT_IDENTITY
  script:
    - dugout run -- ./deploy.sh
```

**Docker:**
```dockerfile
# Secrets injected at runtime, never baked into image
CMD ["dugout", "run", "--", "./start.sh"]
```
```bash
docker run -e DUGOUT_IDENTITY="$KEY" myapp
```

**Any CI/any server:**
```bash
export DUGOUT_IDENTITY="AGE-SECRET-KEY-1..."
dugout run -- ./start.sh
```

## Part 2: KMS Integration (Cloud-Native Upgrade)

### Config

```toml
# .dugout.toml
[meta]
cipher = "age"              # default, always age for team keys

[kms]
# Optional: add KMS as additional decryption path
aws = "arn:aws:kms:us-east-1:123456789012:key/abc-123"
# OR
gcp = "projects/my-proj/locations/global/keyRings/my-ring/cryptoKeys/my-key"
# OR
azure = "https://my-vault.vault.azure.net/keys/my-key"  # future
```

### How Hybrid Works

When KMS is configured:

**Encrypt:** each secret is stored as an envelope:
```json
{
  "version": "dugout-envelope-v1",
  "age": "<age-encrypted for all team member keys>",
  "kms": "<KMS-encrypted ciphertext>"
}
```

**Decrypt:** dugout tries age first (fast, local), falls back to KMS if no age identity is available.

This means:
- Developers decrypt locally with age (fast, offline)
- Production decrypts via KMS API (no key files)
- Both read the exact same `.dugout.toml` vault
- Re-encryption for new team members only touches the `age` portion

### KMS Auth

dugout never manages cloud IAM. It relies on the standard credential chain:

| Provider | Auth method | How it works |
|---|---|---|
| AWS | Default credential chain | Instance role, env vars, ~/.aws/credentials |
| GCP | Application Default Credentials | Service account, metadata server, gcloud auth |
| Azure | DefaultAzureCredential | Managed identity, env vars, az login |

If you can run `aws kms decrypt` on the machine, dugout can too. No extra config.

### CLI Changes for KMS

```bash
# Init with KMS (new flag)
dugout init --kms-key arn:aws:kms:...

# Add KMS to existing project
dugout config set kms.aws "arn:aws:kms:..."

# Re-encrypt all secrets to add KMS envelope
dugout secrets rotate
```

`dugout init --kms-key` auto-detects provider from the key format:
- `arn:aws:kms:` → AWS
- `projects/` → GCP
- `https://*.vault.azure.net` → Azure (future)

## Part 3: Azure KMS (Stub)

Same pattern as AWS/GCP. Feature-gated behind `--features azure`. Uses `azure_security_keyvault` crate. Implement after AWS/GCP are proven.

## Implementation Plan

### Phase 1: Service Identities (cloud-agnostic, no new deps)

1. **Add `DUGOUT_IDENTITY` / `DUGOUT_IDENTITY_FILE` env var support**
   - Modify identity resolution to check env first
   - Add tests: set env var, verify decrypt works
   - ~50 lines of code

2. **Enhance `dugout setup` with `--name` and `--output`**
   - Add CLI flags to setup command
   - `--output -` prints key to stdout
   - `--output <path>` writes key to file
   - Tests: non-interactive setup, output to file, output to stdout
   - ~80 lines of code

3. **Add deployment docs**
   - `docs/deploy.md` — main guide (identity injection, CI examples)
   - README section linking to deploy guide
   - ~200 lines of docs

4. **Integration tests**
   - Test DUGOUT_IDENTITY env var decryption
   - Test DUGOUT_IDENTITY_FILE path decryption
   - Test auth resolution order (env beats file beats global)
   - Test error messages when no identity found
   - ~150 lines of tests

### Phase 2: KMS Wiring (cloud-native)

5. **Wire KMS backends into vault operations**
   - `dugout init --kms-key` flag
   - Auto-detect provider from ARN/resource format
   - Hybrid envelope: encrypt for both age + KMS
   - Decrypt: try age first, fall back to KMS
   - ~200 lines of code

6. **KMS config in .dugout.toml**
   - `[kms]` section with `aws`, `gcp`, `azure` fields
   - `dugout config set kms.aws <arn>` helper
   - Config validation (warn if KMS configured but feature not compiled)
   - ~100 lines of code

7. **Envelope format**
   - Extend existing `WrappedCiphertext` to dual-path envelope
   - Backward compatible: old ciphertexts still decrypt
   - `dugout secrets rotate` re-encrypts to new envelope format
   - ~150 lines of code

8. **KMS integration tests**
   - Mock KMS client for unit tests (no real AWS needed)
   - Test envelope encrypt/decrypt roundtrip
   - Test age-only fallback when KMS unavailable
   - Test KMS-only fallback when no age identity
   - Test hybrid: both paths work on same ciphertext
   - ~200 lines of tests

9. **KMS docs**
   - `docs/kms.md` — setup guide per provider
   - IAM policy examples (AWS, GCP)
   - Update deploy.md with KMS path
   - ~300 lines of docs

### Phase 3: Polish

10. **`dugout check status` improvements**
    - Show active auth method (age key / KMS / both)
    - Show KMS key ARN if configured
    - Show identity source (env var / file / global)
    - ~50 lines

11. **Error messages**
    - No identity found → suggest `dugout setup` or `DUGOUT_IDENTITY`
    - KMS configured but not compiled → suggest `--features aws`
    - KMS auth failed → suggest checking IAM permissions
    - ~30 lines

12. **README deployment section**
    - Brief overview with links to detailed docs
    - One example each: CI (GitHub Actions) and production (Docker)
    - ~50 lines

## Testing Strategy

| Layer | What | How |
|---|---|---|
| Unit | Identity resolution order | Mock env vars, temp files |
| Unit | KMS envelope format | Serialize/deserialize, roundtrip |
| Unit | Provider detection from ARN | Pattern matching tests |
| Integration | DUGOUT_IDENTITY decrypt | Set env, run dugout get |
| Integration | DUGOUT_IDENTITY_FILE decrypt | Write key file, set env, run dugout get |
| Integration | Auth fallback chain | Test each step in order |
| Integration | setup --output | Verify key written to stdout/file |
| Mock | KMS encrypt/decrypt | Mock AWS/GCP client, test full flow |
| Mock | Hybrid envelope | Both decrypt paths on same ciphertext |
| CI | GitHub Actions example | Real workflow in .github/workflows/ |

## File Changes Summary

```
src/cli/setup.rs         — --name, --output flags
src/core/domain/identity — DUGOUT_IDENTITY env var resolution  
src/core/cipher/backend  — hybrid envelope format
src/core/config.rs       — [kms] config section
src/cli/init.rs          — --kms-key flag
tests/identity.rs        — env var resolution tests
tests/deploy.rs          — new: deployment scenario tests
docs/deploy.md           — new: deployment guide
docs/kms.md              — new: KMS setup guide
README.md                — deployment section
```

## Estimated Effort

- Phase 1 (service identities): ~480 lines code/tests + 200 lines docs. **1 session.**
- Phase 2 (KMS wiring): ~650 lines code/tests + 300 lines docs. **1-2 sessions.**
- Phase 3 (polish): ~130 lines. **< 1 session.**

Phase 1 alone covers 90% of deployment use cases. Phase 2 is the premium upgrade.

## Out of Scope

- IAM policy management (terraform's job)
- Key rotation automation (future)
- Multi-region KMS (future)
- Hardware security modules / HSM (future)
- Secret versioning / rollback (future)
