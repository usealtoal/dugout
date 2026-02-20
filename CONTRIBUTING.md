# Contributing to dugout

## Setup

[Rust](https://rustup.rs/) is required to build dugout.

```bash
git clone https://github.com/usemantle/dugout
cd dugout
cargo build
```

## Testing

```bash
cargo test -- --test-threads=1
```

## Benchmarks

```bash
cargo bench
```

## Code Style

- `cargo fmt` before committing
- `cargo clippy -- -D warnings` must pass
- Doc comments on all public items
- Follow existing patterns in the codebase

## Pull Requests

- One concern per PR
- Describe what changed and why
- Add tests for new functionality
- Ensure CI passes

## Reporting Issues

Use [GitHub Issues](https://github.com/usemantle/dugout/issues). Include:

- dugout version (`dugout --version`)
- OS and architecture
- Steps to reproduce
