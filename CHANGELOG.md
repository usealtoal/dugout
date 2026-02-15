# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.8] - 2026-02-15

### Added
- **Multi-vault support** — manage separate secret sets (dev, staging, prod) in the same repository
  - `dugout init --vault <name>` creates named vaults (`.dugout.<name>.toml`)
  - `dugout --vault <name>` flag to select vault for any command
  - `DUGOUT_VAULT` environment variable for vault selection
  - `dugout vault list` to show all vaults in repository
  - Single vault requires no flags; multiple vaults prompt for selection
- Multi-platform release pipeline for Linux (x86_64, aarch64) and macOS (x86_64, aarch64)
- `dugout rotate` command for key rotation with automatic re-encryption
- Enhanced error messages with helpful suggestions and available options
- Comprehensive crates.io metadata for publishing
- **443 tests** including hardening tests for concurrency, fuzzing, and recovery

### Changed
- Error messages now include contextual hints and next steps
- "secret not found" errors show available keys and suggestions
- "not initialized" errors suggest running `dugout init`
- "no private key" errors guide users to either get key from team or init fresh
- CI now runs tests per-category with explicit platform naming

### Security
- Old keys are automatically archived with timestamps during rotation
- Key rotation maintains encryption for all team members
- Rotation plaintext wrapped in `Zeroizing` for secure memory cleanup
- TOCTOU race condition fixed in key rotation archive
- Decryption now has 10MB size limit to prevent memory exhaustion
- Config saves are atomic (temp file + rename) to prevent corruption
- Defense-in-depth validation added to core path functions
- Secret key names demoted from info to debug log level
- Knock request overwrite prevention (detects same vs different key)

## [0.1.0] - 2026-02-07

### Added
- Initial release
- Age-encrypted secret management
- Team collaboration with recipient management
- Git integration with automatic .gitignore setup
- Import/export from .env files
- Lock/unlock workflow for secrets
- Shell integration with `dugout env` and `dugout run`
- Git history audit for leaked secrets
- Status overview and diff commands
- Shell completions for bash, zsh, fish, and PowerShell
- 30+ integration tests
- Comprehensive documentation and security guide

### Security
- Age encryption (X25519) for all secrets
- Private keys stored in ~/.dugout/keys/ with 0600 permissions
- Automatic permission validation on Unix systems
- Zeroizing for sensitive data in memory
- Git history audit to detect leaked secrets

[Unreleased]: https://github.com/usealtoal/dugout/compare/v0.1.8...HEAD
[0.1.8]: https://github.com/usealtoal/dugout/compare/v0.1.0...v0.1.8
[0.1.0]: https://github.com/usealtoal/dugout/releases/tag/v0.1.0
