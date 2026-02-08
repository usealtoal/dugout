# How Dugout Works

A guide to the cryptography, architecture, and design decisions behind dugout.

## The Problem

Every team has secrets: database URLs, API keys, cloud credentials. The standard approach is to toss them in a `.env` file and add it to `.gitignore`. This works until:

- A new developer joins and pings Slack for the keys
- Someone accidentally commits `.env` to git history (it happens constantly)
- Production secrets live in a shared doc, a password manager, or someone's memory
- CI/CD needs secrets, so they end up copy-pasted into dashboard UIs
- Someone leaves the team and you realize they still have every secret

The common "solutions" each have tradeoffs:

| Approach | Problem |
|----------|---------|
| `.env` in `.gitignore` | Not in version control. Shared via Slack, email, prayer |
| Shared password manager | Manual sync. No audit trail. No automation |
| HashiCorp Vault / Doppler | Server infrastructure to maintain. Vendor lock-in. Cost |
| SOPS | Powerful but complex. YAML editing. No team workflow. Steep learning curve |
| dotenvx | No team access control. Tied to their platform for sharing |

Dugout's thesis: **secrets should live in git, encrypted, with access control managed through git commits.** No server. No SaaS. No infrastructure. Just a file in your repo that only your team can read.

## The Cryptography

### age: Actually Good Encryption

Dugout encrypts with [age](https://age-encryption.org), designed by Filippo Valsorda (Google's former Go cryptography lead). It's the modern replacement for GPG — simpler, safer, and with fewer footguns.

Every dugout user has a keypair:

- **Private key** — `AGE-SECRET-KEY-1QFZR...` (an x25519 scalar, stored locally in `~/.dugout/`)
- **Public key** — `age1abc123...` (the corresponding x25519 point, shared in `.dugout.toml`)

The algorithms under the hood:

| Step | Algorithm | Purpose |
|------|-----------|---------|
| Key agreement | X25519 (Curve25519 ECDH) | Derive shared secret between sender and recipient |
| Key wrapping | HKDF + ChaCha20-Poly1305 | Wrap the random file key for each recipient |
| Payload encryption | ChaCha20-Poly1305 | Encrypt the actual secret value |

### Multi-Recipient Encryption

When you run `dugout set DATABASE_URL "postgres://..."`, here's what happens:

1. A random **file key** is generated (256-bit)
2. For **each recipient** in `.dugout.toml`, age performs X25519 ECDH to derive a shared secret, then wraps the file key with that shared secret
3. The plaintext is encrypted with ChaCha20-Poly1305 using the file key
4. The output contains one "stanza" per recipient, plus the encrypted payload

```
-----BEGIN AGE ENCRYPTED FILE-----
[stanza for alice — file key wrapped with alice's shared secret]
[stanza for bob — file key wrapped with bob's shared secret]
[encrypted payload — ChaCha20-Poly1305 with file key]
-----END AGE ENCRYPTED FILE-----
```

Any recipient can independently unwrap the file key using their private key, then decrypt the payload. One ciphertext, multiple readers. Adding a recipient means re-encrypting (so the new person gets a stanza), but the operation is fast — about 100 microseconds per secret.

### Zeroization

Private keys and decrypted secret values are wrapped in `Zeroizing<T>` from the [zeroize](https://crates.io/crates/zeroize) crate. When these values go out of scope, their memory is overwritten with zeros before deallocation. This prevents secrets from lingering in memory after use.

## The Vault

### `.dugout.toml`

The entire state of a dugout project is a single TOML file committed to git:

```toml
[dugout]
version = "0.1.6"
recipients_hash = "a1b2c3d4..."

[recipients]
alice = "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p"
bob = "age1yr4nm5g0wg4f6dekfxqv0c88r4mxrrp0dytt4emz30kgqpcge4tq4f3z7c"

[secrets]
DATABASE_URL = "-----BEGIN AGE ENCRYPTED FILE-----\nYWdlLW..."
API_KEY = "-----BEGIN AGE ENCRYPTED FILE-----\nYWdlLW..."
STRIPE_SECRET = "-----BEGIN AGE ENCRYPTED FILE-----\nYWdlLW..."
```

Three sections:

- **`[dugout]`** — metadata. Version and a SHA-256 fingerprint of the recipient set (used by `sync` to detect when re-encryption is needed)
- **`[recipients]`** — who can decrypt. Maps human names to age public keys
- **`[secrets]`** — the encrypted values. Key names are visible (they're just env var names), values are age ciphertext

This file goes in git. Anyone with repo access can see the secret **names** (`DATABASE_URL`, `API_KEY`) but not the **values**. Only team members with a matching private key can decrypt.

### Identity Resolution

When you run any dugout command, it needs your private key. The resolution order:

1. **`DUGOUT_IDENTITY`** env var — raw private key (for CI/CD pipelines)
2. **`DUGOUT_IDENTITY_FILE`** env var — path to key file (also CI/CD)
3. **Project-local key** — `~/.dugout/keys/<project>/identity.key`
4. **Global key** — `~/.dugout/identity`

After finding a key, dugout checks: is your public key listed in `[recipients]`? If not, you get `AccessDenied`. You can't even attempt decryption without being a recognized team member.

This means access control is enforced at two levels:
- **Cryptographic** — you literally can't decrypt without the right private key
- **Config-level** — even with a valid age key, you're rejected if you're not in `[recipients]`

## The Team Workflow

### Join Flow: knock / admit

```bash
# Bob clones the repo and wants access
bob$ dugout knock
# → writes bob's public key to .dugout/requests/bob.pub
bob$ git add . && git commit -m "requesting access" && git push

# Alice sees the request
alice$ git pull
alice$ dugout pending
# bob  age1yr4nm5g0wg4f6dekfx...

# Alice approves
alice$ dugout admit bob
# → adds bob to [recipients]
# → re-encrypts ALL secrets for [alice, bob]
alice$ git add . && git commit -m "admit bob" && git push

# Bob can now decrypt
bob$ git pull
bob$ dugout get DATABASE_URL
# postgres://...
```

The `admit` step is the critical moment. It re-encrypts every secret so the ciphertext now contains stanzas for both alice AND bob. This is a git commit — fully auditable, reviewable, revertable.

### Leave Flow: team rm

```bash
alice$ dugout team rm bob
# → removes bob from [recipients]
# → re-encrypts ALL secrets for [alice] only
alice$ git add . && git commit -m "remove bob" && git push
```

After this commit, the secrets in git no longer contain a stanza for bob. Even if bob has the old ciphertext (from git history), they can't decrypt the current versions.

### Sync

After pulling changes where the recipient list may have changed:

```bash
dugout sync
```

This compares a SHA-256 fingerprint of the current recipient set against the stored hash. If they differ, all secrets are re-encrypted for the current set. If they match, it's a no-op.

## Hybrid Encryption: age + Cloud KMS

For production environments, dugout supports **hybrid mode** — secrets encrypted for both age keys and cloud KMS.

### The Envelope

When you initialize with `dugout init --kms arn:aws:kms:us-east-1:...`, secrets are stored as JSON envelopes instead of raw age ciphertext:

```json
{
  "v": 2,
  "age": "-----BEGIN AGE ENCRYPTED FILE-----\n...",
  "kms": "AQIBAHh...<base64 KMS ciphertext>",
  "provider": "aws"
}
```

Each secret is encrypted **twice**: once with age (for developers), once with KMS (for production). Both ciphertexts are stored together in the envelope.

### Decrypt Paths

On decryption, dugout tries the fast path first:

1. **age identity available?** → decrypt via age (instant, offline, no network)
2. **No age identity, but cloud credentials?** → decrypt via KMS API call

Developers hit path 1. CI/CD pipelines and production servers hit path 2 via IAM roles — no key files needed.

### Why Both?

| Path | Speed | Requires | Use case |
|------|-------|----------|----------|
| age | ~135µs | Private key file | Local development |
| KMS | ~50ms | IAM role/credentials | CI/CD, production |

Age is fast and works offline. KMS integrates with cloud IAM (no key distribution). Having both means the same vault works everywhere without configuration changes.

### Backward Compatibility

- A hybrid backend can read raw age ciphertext (pre-KMS secrets just work)
- An age-only backend can read hybrid envelopes (it ignores the KMS field and decrypts the age portion)

This means you can upgrade to hybrid mode without re-encrypting existing secrets, and team members without the `aws` or `gcp` feature compiled can still decrypt via age.

## Running With Secrets

### dugout run

```bash
dugout run -- npm start
```

This decrypts all secrets, injects them as environment variables into the child process, and runs the command. Secrets never touch disk — they go directly from encrypted TOML to process memory to child environment.

### dugout . (dot)

```bash
dugout .
```

Auto-detects your project type and runs the appropriate dev command:

| Detected file | Stack | Command |
|--------------|-------|---------|
| `package.json` + bun.lockb | Bun | `bun run dev` |
| `package.json` | Node | `npm run dev` |
| `Cargo.toml` | Rust | `cargo run` |
| `pyproject.toml` | Python | `uv run python -m <pkg>` |
| `go.mod` | Go | `go run .` |
| `Gemfile` | Ruby | `bundle exec rails server` |

One command. No configuration. Secrets are injected automatically.

## How It Compares

### vs SOPS

SOPS encrypts entire files (YAML, JSON, ENV) and stores them in git. It's powerful but complex — you edit encrypted files in-place, configure `.sops.yaml` with path rules and key groups, and manage key rotation manually. There's no team workflow; adding someone means giving them a key and hoping the encryption rules are right.

Dugout is opinionated where SOPS is flexible. One file (`.dugout.toml`), one format, built-in team management (`knock`/`admit`), and commands that map to what you actually want to do (`set`, `get`, `sync`). The tradeoff: SOPS handles arbitrary file encryption, dugout handles key-value secrets.

### vs dotenvx

dotenvx encrypts `.env` files with a single key. Simple and effective for solo use, but sharing requires distributing that key. There's no per-user access control — everyone shares one decryption key, so you can't revoke one person's access without rotating for everyone.

Dugout gives each team member their own keypair. Revoking access means removing their key and re-encrypting — the crypto enforces it.

### vs HashiCorp Vault / Doppler / Infisical

These are server-based solutions. They're excellent for large organizations that need centralized policy, dynamic secrets, and compliance features. They're also infrastructure you have to run, monitor, and pay for.

Dugout is for teams that want secrets in version control with zero infrastructure. If you're a 2-50 person team and your secrets are environment variables, dugout is probably all you need. If you need LDAP integration, dynamic database credentials, or SOC2 audit logs — use Vault.

## Architecture

```
CLI Layer (src/cli/)
├── setup.rs, init.rs, whoami.rs    # Identity & vault setup
├── secrets/                         # set, get, rm, list, lock, unlock, import, export, diff, rotate
├── team/                            # add, list, rm
├── check/                           # status, audit
├── sync.rs, knock.rs, admit.rs     # Team workflow
├── run.rs, dot.rs, shell.rs        # Execution with secrets
└── output.rs                        # Minimal output helpers

Core Layer (src/core/)
├── vault.rs                         # Primary API — all operations go through Vault
├── config.rs                        # .dugout.toml read/write/validate
├── detect.rs                        # Project type detection
├── domain/                          # Pure types: Secret, Identity, Recipient, Diff, Env, SyncResult
├── cipher/                          # Encryption backends
│   ├── age.rs                       # age encryption (default)
│   ├── backend.rs                   # CipherBackend dispatch (Age | Hybrid)
│   ├── envelope.rs                  # v2 envelope format, KMS provider detection
│   ├── aws.rs                       # AWS KMS (feature-gated)
│   └── gcp.rs                       # GCP KMS (feature-gated)
└── store/                           # Key storage & file operations
```

The CLI layer is thin — it parses arguments, calls `Vault` methods, and formats output. All logic lives in `Vault`, which owns the config, manages keys, and delegates to the cipher backend.

The cipher layer has a single trait (`Cipher`) and two backends (`Age`, `Hybrid`). Adding a new backend means implementing the trait — the rest of the system doesn't change.

Output follows the "silent success" pattern: success is one line, errors are one line plus a hint, data output is raw and pipeable. No spinners, no progress bars, no noise.
