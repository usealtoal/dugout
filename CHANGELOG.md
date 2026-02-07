# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Multi-platform release pipeline for Linux (x86_64, aarch64) and macOS (x86_64, aarch64)
- `burrow rotate` command for key rotation with automatic re-encryption
- Enhanced error messages with helpful suggestions and available options
- Comprehensive crates.io metadata for publishing

### Changed
- Error messages now include contextual hints and next steps
- "secret not found" errors show available keys and suggestions
- "not initialized" errors suggest running `burrow init`
- "no private key" errors guide users to either get key from team or init fresh

### Security
- Old keys are automatically archived with timestamps during rotation
- Key rotation maintains encryption for all team members

## [0.1.0] - 2026-02-07

### Added
- Initial release
- Age-encrypted secret management
- Team collaboration with recipient management
- Git integration with automatic .gitignore setup
- Import/export from .env files
- Lock/unlock workflow for secrets
- Shell integration with `burrow env` and `burrow run`
- Git history audit for leaked secrets
- Status overview and diff commands
- Shell completions for bash, zsh, fish, and PowerShell
- 30+ integration tests
- Comprehensive documentation and security guide

### Security
- Age encryption (X25519) for all secrets
- Private keys stored in ~/.burrow/keys/ with 0600 permissions
- Automatic permission validation on Unix systems
- Zeroizing for sensitive data in memory
- Git history audit to detect leaked secrets

[Unreleased]: https://github.com/usealtoal/burrow/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/usealtoal/burrow/releases/tag/v0.1.0
