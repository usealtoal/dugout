# Contributing to burrow

## Setup

[Rust](https://rustup.rs/) is required to build burrow.

```bash
git clone https://github.com/usealtoal/burrow
cd burrow
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
- See [STYLE.md](STYLE.md) for conventions

## Pull Requests

- One concern per PR
- Describe what changed and why
- Add tests for new functionality
- Ensure CI passes

## Reporting Issues

Use [GitHub Issues](https://github.com/usealtoal/burrow/issues). Include:

- burrow version (`burrow --version`)
- OS and architecture
- Steps to reproduce
