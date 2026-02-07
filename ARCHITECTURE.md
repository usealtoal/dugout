# Architecture

## Overview

Burrow is a local-first secrets manager built on [age encryption](https://age-encryption.org/). It stores encrypted secrets in a TOML file that lives in your git repo, decrypting them only when needed.

## Core Design

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   CLI Layer  │────▶│  Core Engine  │────▶│  age Crypto │
│  (clap)      │     │  (encrypt/   │     │  (X25519 +  │
│              │     │   decrypt)   │     │  ChaCha20)  │
└─────────────┘     └──────────────┘     └─────────────┘
       │                    │
       ▼                    ▼
┌─────────────┐     ┌──────────────┐
│   Config     │     │  Key Store   │
│ .burrow.toml │     │ ~/.burrow/   │
│ (committed)  │     │ (local only) │
└─────────────┘     └──────────────┘
```

## Module Layout

```
src/
├── main.rs          # Entry point, CLI dispatch
├── cli.rs           # Command definitions (clap derive)
├── config.rs        # .burrow.toml read/write
├── crypto.rs        # age encrypt/decrypt operations
├── keystore.rs      # Key management (~/.burrow/keys/)
├── secrets.rs       # Secret CRUD operations
├── runner.rs        # `burrow run` subprocess with env injection
├── team.rs          # Team member (recipient) management
├── import_export.rs # .env import/export
└── error.rs         # Error types
```

## Encryption Flow

### Set a secret
1. Read `.burrow.toml` to get recipient public keys
2. Encrypt value with age for all recipients
3. Store encrypted blob in `.burrow.toml` under `[secrets]`
4. Write updated TOML

### Unlock (decrypt to .env)
1. Read `.burrow.toml` encrypted secrets
2. Load private key from `~/.burrow/keys/<project>/`
3. Decrypt each secret with age
4. Write plaintext `.env` (gitignored)

### Run with secrets
1. Same as unlock, but in-memory only
2. Inject as environment variables
3. Exec the child process
4. Secrets never touch disk

## Security Model

- Private keys never leave `~/.burrow/keys/`
- `.burrow.toml` contains only encrypted data (safe to commit)
- `.env` is always in `.gitignore`
- No network calls ever
- `burrow run` keeps secrets in-memory only
