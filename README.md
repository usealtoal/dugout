<div align="center">
  <pre>
  ██████  ██    ██ ██████  ██████   ██████  ██     ██
  ██   ██ ██    ██ ██   ██ ██   ██ ██    ██ ██     ██
  ██████  ██    ██ ██████  ██████  ██    ██ ██  █  ██
  ██   ██ ██    ██ ██   ██ ██   ██ ██    ██ ██ ███ ██
  ██████   ██████  ██   ██ ██   ██  ██████   ███ ███
  </pre>

  <p><strong>An extremely fast secrets manager for developers, written in Rust</strong></p>

  <p>
    <img src="https://img.shields.io/badge/rust-1.75%2B-orange.svg" alt="Rust">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
    <img src="https://img.shields.io/badge/status-alpha-yellow.svg" alt="Status">
  </p>

  <p><em>Dig deep. Ship safe.</em> 🐀</p>
</div>

---

## What is Burrow?

Burrow is a CLI tool that makes secrets management simple. Encrypt your `.env` files, commit them to git, and share them with your team. No SaaS, no cloud dependency, no vendor lock-in.

Think **SOPS simplicity** meets **Doppler UX**, running locally at Rust speed.

## Why Burrow?

| Problem | Other Tools | Burrow |
|---------|------------|--------|
| `.env` files in `.gitignore` | Every project ever | Encrypt and commit them |
| SOPS is powerful but complex | SOPS, age, GPG | One command to encrypt |
| SaaS secrets managers need internet | Doppler, Infisical | Works offline, always |
| Sharing secrets is a mess | Slack DMs, 1Password | Git push, git pull |
| Onboarding takes forever | "Ask Dave for the keys" | `burrow init`, done |

### vs. Alternatives

| Feature | Burrow | SOPS | Doppler | dotenv-vault |
|---------|--------|------|---------|-------------|
| Speed | Rust-fast | Go | Cloud latency | Node |
| Offline | ✅ Yes | ✅ Yes | ❌ No | ⚠️ Partial |
| Git-friendly | ✅ Yes | ✅ Yes | ❌ No | ✅ Yes |
| Easy setup | 1 command | Complex | Easy | Easy |
| Team sharing | age keys | KMS/PGP | Dashboard | Cloud |
| Vendor lock-in | None | None | Yes | Yes |
| Cost | Free | Free | Freemium | Freemium |

## Quick Start

```bash
# Install
cargo install burrow

# Initialize in your project
cd my-project
burrow init

# Add secrets
burrow set DATABASE_URL "postgres://localhost/myapp"
burrow set API_KEY "sk-live-abc123"

# Encrypt and commit
burrow secrets lock
git add .burrow.toml
git commit -m "add encrypted secrets"

# On another machine (after git pull)
burrow secrets unlock
# Creates .env file with decrypted secrets

# Or run commands with secrets injected
burrow run npm start
```

## How It Works

1. **`burrow init`** generates an age keypair. Public key goes in `.burrow.toml` (committed). Private key goes in `~/.burrow/keys/` (never committed).
2. **`burrow set KEY VALUE`** stores secrets in `.burrow.toml`, encrypted with your project's public key.
3. **`burrow secrets lock`** ensures all secrets are encrypted (re-encrypts any plaintext).
4. **`burrow secrets unlock`** decrypts secrets to a local `.env` file (automatically gitignored).
5. **`burrow run <command>`** injects secrets as environment variables and runs your command.

Secrets are encrypted with [age](https://age-encryption.org/) (modern, audited, simple). Multiple team members can be added as recipients, each with their own private key.

## Commands

### Core Operations

#### `burrow init`
Initialize burrow in the current directory. Generates an age keypair, creates `.burrow.toml`, and updates `.gitignore`.

```bash
burrow init
```

#### `burrow set <KEY> <VALUE>`
Set a secret. Encrypts the value for all configured recipients.

```bash
burrow set DATABASE_URL "postgres://user:pass@localhost/db"
burrow set API_KEY "sk-live-abc123"

# Force overwrite existing secret
burrow set --force DATABASE_URL "postgres://new-host/db"
```

#### `burrow get <KEY>`
Get a secret value (decrypted).

```bash
burrow get DATABASE_URL
# Output: postgres://user:pass@localhost/db
```

#### `burrow rm <KEY>`
Remove a secret.

```bash
burrow rm API_KEY
```

#### `burrow list`
List all secret keys (names only, not values).

```bash
burrow list
# Output:
# API_KEY
# DATABASE_URL
# STRIPE_SECRET
```

### Secret Lifecycle

#### `burrow secrets lock`
Encrypt all secrets. Ensures nothing is stored in plaintext.

```bash
burrow secrets lock
```

#### `burrow secrets unlock`
Decrypt all secrets to `.env` file.

```bash
burrow secrets unlock
```

#### `burrow secrets import <file>`
Import secrets from a `.env` file.

```bash
burrow secrets import .env
burrow secrets import production.env
```

#### `burrow secrets export`
Export decrypted secrets in `.env` format.

```bash
burrow secrets export
```

#### `burrow secrets diff`
Show differences between encrypted secrets and local `.env` file.

```bash
burrow secrets diff
```

#### `burrow secrets rotate`
Rotate the project encryption key. Generates a new keypair and re-encrypts all secrets.

```bash
burrow secrets rotate
```

### Running Commands

#### `burrow run <command>`
Run a command with secrets injected as environment variables.

```bash
burrow run npm start
burrow run cargo test
burrow run python manage.py migrate
```

#### `burrow env`
Start an interactive shell with secrets loaded.

```bash
burrow env
```

### Team Management

#### `burrow team add <name> <public-key>`
Add a team member. All secrets will be re-encrypted for the new recipient.

```bash
burrow team add alice age1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqsz0

# Or import from file
burrow team add bob ~/.burrow/keys/bob.pub
```

#### `burrow team list`
List all team members and their public keys.

```bash
burrow team list
```

#### `burrow team remove <name>`
Remove a team member. All secrets will be re-encrypted without this recipient.

```bash
burrow team remove alice
```

### Import/Export

#### `burrow import <file>`
Import secrets from a `.env` file.

```bash
burrow import .env
burrow import production.env
```

#### `burrow export`
Export decrypted secrets in `.env` format.

```bash
burrow export
burrow export --output .env.backup
```

#### `burrow diff`
Show differences between encrypted secrets and local `.env` file.

```bash
burrow diff
```

### Utilities

#### `burrow status`
Show quick status overview (number of secrets, team members, etc.).

```bash
burrow status
```

#### `burrow rotate`
Rotate the project encryption key. Generates a new keypair and re-encrypts all secrets.

```bash
burrow rotate
```

#### `burrow audit`
Scan git history for accidentally committed plaintext secrets.

```bash
burrow audit
```

#### `burrow completions <shell>`
Generate shell completions.

```bash
# Bash
burrow completions bash > /etc/bash_completion.d/burrow

# Zsh
burrow completions zsh > ~/.zsh/completions/_burrow

# Fish
burrow completions fish > ~/.config/fish/completions/burrow.fish
```

## Configuration

`.burrow.toml` lives in your project root and gets committed to git:

```toml
[burrow]
version = "0.1.0"

[recipients]
rob = "age1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqsz0"
alice = "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p"

[secrets]
DATABASE_URL = "-----BEGIN AGE ENCRYPTED FILE-----\nYWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBE...\n-----END AGE ENCRYPTED FILE-----"
API_KEY = "-----BEGIN AGE ENCRYPTED FILE-----\nYWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBR...\n-----END AGE ENCRYPTED FILE-----"
```

### Key Storage

Private keys are stored in `~/.burrow/keys/` and organized by project:

```
~/.burrow/
└── keys/
    ├── my-project.key        # Project private key
    ├── other-project.key
    └── ...
```

Keys are named after the project directory. Never commit these files.

## Security

Burrow supports multiple cipher backends for encryption:

### Cipher Backends

#### age (default)

Uses [age](https://age-encryption.org/) for encryption - always available, no feature flags needed.

- **Algorithm:** X25519 (key agreement) + ChaCha20-Poly1305 (encryption)
- **Key size:** 256-bit
- **Authenticated encryption:** Yes (AEAD)
- **Best for:** General use, offline-first workflows, developer teams

```bash
burrow init  # Uses age by default
```

#### AWS KMS

Enterprise encryption using AWS Key Management Service. Enable with `--features aws`.

- **Algorithm:** Configurable (AES-256-GCM, RSA, etc.)
- **Key management:** AWS manages keys in the cloud
- **Best for:** AWS-native environments, compliance requirements, central key management
- **Requires:** AWS credentials (via env vars or AWS config)

```bash
# Build with AWS support
cargo install burrow --features aws

# Initialize with AWS KMS
burrow init --cipher aws-kms --kms-key arn:aws:kms:us-east-1:123456:key/abc-def

# Or update existing vault
# .burrow.toml:
[meta]
cipher = "aws-kms"

[meta.kms]
key_id = "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012"
```

#### GCP KMS

Google Cloud Key Management Service via gcloud CLI. Enable with `--features gcp`.

- **Algorithm:** Configurable (AES-256-GCM, RSA, etc.)
- **Key management:** GCP manages keys in the cloud
- **Best for:** GCP-native environments, Google Workspace integration
- **Requires:** `gcloud` CLI installed and authenticated

```bash
# Build with GCP support
cargo install burrow --features gcp

# Initialize with GCP KMS
burrow init --cipher gcp-kms --gcp-key projects/my-project/locations/global/keyRings/my-ring/cryptoKeys/my-key

# Or update existing vault
# .burrow.toml:
[meta]
cipher = "gcp-kms"

[meta.gcp]
resource_name = "projects/my-project/locations/global/keyRings/my-ring/cryptoKeys/my-key"
```

#### GPG

Uses GnuPG for OpenPGP-compatible encryption. Enable with `--features gpg`.

- **Algorithm:** Configurable (RSA, ECC, etc.)
- **Key management:** Local GPG keyring
- **Best for:** Organizations already using GPG/PGP, compliance with PGP standards
- **Requires:** `gpg` CLI installed with keys in your keyring

```bash
# Build with GPG support
cargo install burrow --features gpg

# Initialize with GPG
burrow init --cipher gpg

# Recipients are GPG key fingerprints or email addresses
burrow team add alice alice@example.com
burrow team add bob ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234

# Or update existing vault
# .burrow.toml:
[meta]
cipher = "gpg"

[recipients]
alice = "alice@example.com"
bob = "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234"
```

#### Multi-Backend Builds

Enable multiple backends at once:

```bash
cargo install burrow --features aws,gcp,gpg
```

### Security Properties

- **Encrypted at rest:** Secrets in `.burrow.toml` are always encrypted
- **No network calls:** Everything happens locally (except cloud KMS backends)
- **No telemetry:** Zero tracking or analytics
- **Memory safety:** Rust + zeroization of decrypted values
- **Multi-recipient:** Each team member has their own private key (age, GPG)

### Best Practices

1. **Never commit `.env` files** - Burrow automatically adds them to `.gitignore`
2. **Protect private keys** - Store in `~/.burrow/keys/`, never commit
3. **Use `burrow secrets lock`** - Ensures all secrets are encrypted before committing
4. **Audit regularly** - Run `burrow check audit` to scan for leaked secrets
5. **Rotate on compromise** - If a key is leaked, run `burrow secrets rotate` and remove the compromised recipient

For vulnerability reports, see [SECURITY.md](SECURITY.md).

## Team Workflow Example

### Initial Setup (by project lead)

```bash
# Create project and initialize burrow
mkdir my-app
cd my-app
git init
burrow init

# Add secrets
burrow set DATABASE_URL "postgres://localhost/myapp"
burrow set API_KEY "sk-live-abc123"

# Commit encrypted config
git add .burrow.toml .gitignore
git commit -m "add encrypted secrets"
git push
```

### Adding a Team Member

Alice joins the team and shares her public key:

```bash
# Alice generates her key (on her machine)
burrow init --export > alice.pub

# Project lead adds Alice
burrow team add alice $(cat alice.pub)

# All secrets are re-encrypted for Alice
git add .burrow.toml
git commit -m "add Alice to team"
git push
```

### New Team Member Setup

Alice clones the repo and can immediately decrypt secrets:

```bash
# Alice clones and unlocks
git clone git@github.com:company/my-app.git
cd my-app
burrow secrets unlock

# Secrets are now in .env (gitignored)
burrow run npm start
```

### Removing a Team Member

When Bob leaves:

```bash
# Remove Bob's access
burrow team remove bob

# All secrets are re-encrypted without Bob
git add .burrow.toml
git commit -m "remove Bob from team"
git push
```

Bob can no longer decrypt new or updated secrets (but retains access to secrets he previously decrypted).

## Performance

Burrow is fast. Benchmarks on a modern laptop:

| Operation | 32B payload | 1KB payload | 4KB payload |
|-----------|-------------|-------------|-------------|
| Encrypt (1 recipient) | ~50 µs | ~60 µs | ~80 µs |
| Decrypt | ~40 µs | ~45 µs | ~60 µs |
| Roundtrip (3 recipients) | ~120 µs | ~140 µs | ~180 µs |
| Config save (20 secrets) | ~2 ms | ~2 ms | ~2 ms |

Run benchmarks yourself:

```bash
cargo bench
```

## Contributing

Contributions are welcome! Please:

1. Fork the repo
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Make your changes
4. Run tests (`cargo test`)
5. Run clippy (`cargo clippy -- -D warnings`)
6. Format code (`cargo fmt`)
7. Commit (`git commit -am 'Add amazing feature'`)
8. Push (`git push origin feature/amazing`)
9. Open a Pull Request

## License

MIT License. See [LICENSE](LICENSE) for details.

---

<div align="center">
  <p><strong>Built with Rust. Secured with age. Trusted by developers.</strong></p>
  <p>⭐ Star us on GitHub if burrow helps you ship faster!</p>
</div>
