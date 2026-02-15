<p align="center">
  <img alt="dugout" src="assets/mole-banner.png" width="600">
</p>

<p align="center">
  <a href="https://github.com/usealtoal/dugout/actions"><img src="https://github.com/usealtoal/dugout/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://crates.io/crates/dugout"><img src="https://img.shields.io/crates/v/dugout.svg" alt="Crates.io"></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg" alt="License"></a>
</p>

<p align="center">
  <strong>Git-native secrets manager for development teams, written in Rust.</strong>
</p>

## Highlights

- **Git-native** — secrets live in your repo as encrypted values, access control is git commits
- **No server required** — no SaaS, no cloud dependency, no infrastructure to manage
- **Team-friendly** — `knock` / `admit` workflow for access requests, all through git
- **Encrypted at rest** — age encryption by default, optional AWS KMS, GCP KMS
- **Zero config** — `dugout init` and start adding secrets
- **Auto-detect** — `dugout .` detects your stack and runs with secrets injected
- **Fast** — encrypts in ~100µs, single binary, no runtime dependencies
- **Vendor-agnostic** — works with any git host, any infrastructure, any language

## Comparison

| | dugout | sops | dotenvx | Vault | Doppler | Infisical |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| **Secrets in repo** | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| **No server** | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| **No config file** | ✅ | ❌ | ✅ | ❌ | ✅ | ❌ |
| **Team access via git** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Auto-detect & run** | ✅ | ❌ | ✅ | ❌ | ✅ | ❌ |
| **Single binary** | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Encrypt speed** | ~100µs | ~1ms | N/A | N/A | N/A | N/A |
| **Free & open source** | ✅ | ✅ | ✅ | ✅* | ❌ | ✅ |
| **Written in** | Rust | Go | JS | Go | — | TS |

<sup>*Vault BSL license</sup>

## Installation

Install dugout with our standalone installers:

```bash
# On macOS and Linux.
curl -LsSf https://raw.githubusercontent.com/usealtoal/dugout/main/scripts/install.sh | sh
```

```bash
# On Windows.
powershell -ExecutionPolicy ByPass -c "irm https://raw.githubusercontent.com/usealtoal/dugout/main/scripts/install.ps1 | iex"
```

Or, with [Homebrew](https://brew.sh):

```bash
brew install usealtoal/tap/dugout
```

Or, from [crates.io](https://crates.io/crates/dugout):

```bash
cargo install dugout
```

Or, from source:

```bash
git clone https://github.com/usealtoal/dugout && cd dugout
cargo install --path .
```

## Quick Start

```bash
# One-time identity setup
dugout setup

# Initialize in your project
cd my-app
dugout init

# Add secrets
dugout set DATABASE_URL postgres://localhost/db
dugout set STRIPE_KEY sk_live_xxx

# Run your app with secrets (auto-detect)
dugout .

# Or run any command with secrets injected
dugout run -- npm start
dugout run -- python manage.py runserver
dugout run -- cargo run
```

## Team Workflow

```bash
# Alice creates the project
dugout init
dugout set API_KEY sk_live_xxx
git add .dugout.toml && git commit -m "init vault" && git push

# Bob clones and requests access
git clone ... && cd project
dugout knock
git add .dugout/requests/ && git commit -m "request access" && git push

# Alice approves
git pull
dugout admit bob
git commit -am "grant bob access" && git push

# Bob pulls and runs
git pull
dugout .
```

No Slack DMs. No shared password vaults. No `.env` files in git history. Access requests and approvals are git commits.

## Commands

| Command | Description |
|---------|-------------|
| `dugout setup` | Generate global identity |
| `dugout init` | Initialize vault in current directory |
| `dugout set KEY VALUE` | Set a secret |
| `dugout get KEY` | Get a secret value |
| `dugout add KEY` | Add a secret interactively |
| `dugout list` | List all secret keys |
| `dugout rm KEY` | Remove a secret |
| `dugout .` | Auto-detect project and run with secrets |
| `dugout run -- CMD` | Run a command with secrets injected |
| `dugout knock` | Request vault access |
| `dugout admit NAME` | Approve an access request |
| `dugout pending` | List pending requests |
| `dugout team add/rm/list` | Manage team members |
| `dugout secrets diff` | Compare vault and .env |
| `dugout secrets rotate` | Rotate encryption keys |
| `dugout secrets lock/unlock` | Lock or decrypt secrets |
| `dugout secrets import/export` | Import or export .env files |
| `dugout check status` | Vault overview |
| `dugout check audit` | Audit for leaked secrets |

## Cipher Backends

| Backend | Flag | Use Case |
|---------|------|----------|
| **age** (default) | — | Local development, small teams |
| AWS KMS | `--features aws` | AWS infrastructure, compliance requirements |
| GCP KMS | `--features gcp` | Google Cloud infrastructure |

```bash
# Initialize with hybrid encryption (age + KMS)
dugout init --kms arn:aws:kms:us-east-1:...

# Install with AWS KMS support
cargo install dugout --features aws
```

See the full [KMS Integration Guide](KMS.md) for AWS, GCP, IAM setup, and multi-region.

## CI/CD

### GitHub Actions

```yaml
- uses: usealtoal/setup-dugout@v1
  with:
    identity: ${{ secrets.DUGOUT_IDENTITY }}

- run: dugout run -- npm test
```

See [`usealtoal/setup-dugout`](https://github.com/usealtoal/setup-dugout) for version pinning, KMS-only mode, and more examples.

### Other environments

```bash
# Any CI — just set the env var
export DUGOUT_IDENTITY="AGE-SECRET-KEY-1..."
dugout run -- ./deploy.sh

# Docker
docker run -e DUGOUT_IDENTITY="$KEY" myapp
```

See the full [Deployment Guide](DEPLOY.md) for GitLab, Kubernetes, and more.

## Benchmarks

Measured with [Criterion](https://github.com/bheisler/criterion.rs). See [BENCHMARKS.md](BENCHMARKS.md) for methodology.

| Operation | 32B | 4KB | 16KB |
|-----------|-----|-----|------|
| Encrypt | 105µs | 113µs | 138µs |
| Decrypt | 135µs | 154µs | 195µs |
| Roundtrip | 258µs | 271µs | 355µs |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup and guidelines.

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
