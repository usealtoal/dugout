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

## What is Burrow?

Burrow is a CLI tool that makes secrets management simple. Encrypt your `.env` files, commit them to git, and share them with your team. No SaaS, no cloud dependency, no vendor lock-in.

Think **SOPS simplicity** meets **Doppler UX**, running locally at Rust speed.

## Why Burrow?

| Problem | Existing Tools | Burrow |
|---------|---------------|--------|
| `.env` files in `.gitignore` | Every project ever | Encrypt and commit them |
| SOPS is powerful but complex | SOPS, age, GPG | One command to encrypt |
| SaaS secrets managers need internet | Doppler, Infisical | Works offline, always |
| Sharing secrets is a mess | Slack DMs, 1Password | Git push, git pull |
| Onboarding takes forever | "Ask Dave for the keys" | `burrow init`, done |

## Quick Start

```bash
# Install
cargo install burrow

# Initialize in your project
burrow init

# Add secrets
burrow set DATABASE_URL "postgres://localhost/myapp"
burrow set API_KEY "sk-live-abc123"

# Encrypt and commit
burrow lock
git add .burrow.toml
git commit -m "add encrypted secrets"

# On another machine
git pull
burrow unlock
```

## How It Works

1. `burrow init` generates an age keypair. Public key goes in `.burrow.toml` (committed). Private key goes in `~/.burrow/keys/` (never committed).
2. `burrow set KEY VALUE` stores secrets in `.burrow.toml`, encrypted with your project's public key.
3. `burrow lock` encrypts any plaintext secrets.
4. `burrow unlock` decrypts to a local `.env` file (gitignored).
5. `burrow run <command>` injects secrets as env vars and runs your command.

Secrets are encrypted with [age](https://age-encryption.org/) (modern, audited, simple). Multiple team members can be added as recipients.

## Commands

```
burrow init              Initialize burrow in current directory
burrow set <KEY> <VAL>   Set a secret
burrow get <KEY>         Get a secret value
burrow rm <KEY>          Remove a secret
burrow list              List all secret keys
burrow lock              Encrypt all secrets
burrow unlock            Decrypt to local .env
burrow run <cmd>         Run command with secrets injected
burrow team add <key>    Add a team member's public key
burrow team list         List team members
burrow diff              Show changes since last lock
burrow export            Export as .env format
burrow import <file>     Import from .env file
```

## Configuration

`.burrow.toml` lives in your project root and gets committed to git:

```toml
[burrow]
version = "0.1.0"

[recipients]
rob = "age1abc123..."
chud = "age1def456..."

[secrets]
DATABASE_URL = "age:encrypted-blob-here..."
API_KEY = "age:encrypted-blob-here..."
```

## Security

- Encryption: age (X25519 + ChaCha20-Poly1305)
- Keys: Stored in `~/.burrow/keys/`, never committed
- Secrets: Encrypted at rest in git, decrypted only locally
- No network calls, no telemetry, no cloud

## vs. Alternatives

| Feature | Burrow | SOPS | Doppler | dotenv-vault |
|---------|--------|------|---------|-------------|
| Speed | Rust-fast | Go | Cloud latency | Node |
| Offline | Yes | Yes | No | Partial |
| Git-friendly | Yes | Yes | No | Yes |
| Easy setup | 1 command | Complex | Easy | Easy |
| Team sharing | age keys | KMS/PGP | Dashboard | Cloud |
| Vendor lock-in | None | None | Yes | Yes |
| Free | Yes | Yes | Freemium | Freemium |

## License

MIT
</div>
