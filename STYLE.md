# Burrow CLI Style Guide

This document defines the output conventions for burrow's command-line interface to ensure a consistent, professional user experience.

## Philosophy

Burrow's CLI output follows these principles:

1. **Beautiful by default** - Use color and formatting to create a polished experience
2. **Scriptable when needed** - Plain output for commands that should be parsed (e.g., `burrow get`)
3. **Respectful** - Honor `NO_COLOR` environment variable for accessibility
4. **Informative** - Clear, concise feedback without unnecessary verbosity
5. **Consistent** - Uniform patterns across all commands

## Color Scheme

Burrow follows the color conventions popularized by modern CLI tools like `uv` and `ruff`:

| Color | Usage | Examples |
|-------|-------|----------|
| **Green** | Success, checkmarks | `✓ initialized`, `burrow set KEY VALUE` |
| **Red** | Errors | `✗ key not found` |
| **Yellow** | Warnings | `⚠ key already exists` |
| **Cyan** | Hints, paths, commands, keys | `→ run burrow unlock`, `.burrow.toml`, `DATABASE_URL` |
| **Bold** | Headers, important values | Section titles, counts |
| **Dimmed** | Secondary info, labels | Key-value labels, empty states |

## Output Helpers

All CLI output should use the helpers in `src/cli/output.rs`:

```rust
use crate::cli::output;

// Success messages
output::success("initialized");  // ✓ initialized

// Errors (to stderr)
output::error("key not found");  // ✗ key not found

// Warnings
output::warn("key already exists");  // ⚠ key already exists

// Hints
output::hint("run burrow unlock to decrypt");  // → run burrow unlock to decrypt

// Section headers with separator
output::section("Configuration");

// Key-value pairs
output::kv("recipient", "alice");  // label dimmed, value bold

// List items
output::list_item("DATABASE_URL");  // • DATABASE_URL

// Progress indicators
output::progress("Encrypting");
// ... do work ...
output::progress_done(true);  // ok (green) or failed (red)

// Inline formatting
println!("Next: {}", output::cmd("burrow unlock"));  // green command
println!("Edit {}", output::path(".burrow.toml"));  // cyan path
println!("Set {}", output::key("DATABASE_URL"));    // cyan key
```

## Command Output Patterns

### Initialization (`burrow init`)

```text
✓ burrow initialized
  recipient:  alice (age1qyq2z3x...)
  config:     .burrow.toml (commit this)
  key:        ~/.burrow/keys/abc123/

→ Next: burrow set KEY VALUE to add secrets
```

### Setting secrets (`burrow set KEY VALUE`)

```text
✓ set: DATABASE_URL
```

### Getting secrets (`burrow get KEY`)

```text
postgres://localhost/db
```

**Important:** No decoration, no colors - just the raw value for scripting.

### Listing secrets (`burrow list`)

```text
3 secrets
────────────────────────────────────────────────────────
  • API_KEY
  • DATABASE_URL
  • SECRET_TOKEN
```

Empty state:
```text
no secrets stored
```

### Removing secrets (`burrow rm KEY`)

```text
✓ removed: DATABASE_URL
```

### Locking (`burrow lock`)

```text
Checking encryption... ok
✓ locked: 3 secrets encrypted in .burrow.toml
  status:  safe to commit
```

### Unlocking (`burrow unlock`)

```text
Decrypting secrets... ok
✓ unlocked: 3 secrets written to .env
```

### Team management (`burrow team list`)

```text
2 team members
────────────────────────────────────────────────────────
  alice:  age1qyq2z3x4y5z6a7b8c9...
  bob:    age1abc123def456ghi789...
```

### Importing (`burrow import .env`)

```text
✓ imported 3 secrets from .env
  • API_KEY
  • DATABASE_URL
  • SECRET_TOKEN
```

### Status/diff (`burrow diff`)

```text
Status
────────────────────────────────────────────────────────
  .burrow.toml:  3 secrets (encrypted)
  .env:          3 entries (plaintext)
```

## Error Handling

All errors should go through `output::error()` which:
- Prints to **stderr** (not stdout)
- Uses red color with `✗` symbol
- Preserves error message detail

```rust
if let Err(e) = do_something() {
    output::error(&e.to_string());
    std::process::exit(1);
}
```

## NO_COLOR Support

The `output` module automatically respects the `NO_COLOR` environment variable:

```bash
# With colors (default)
$ burrow list
✓ 3 secrets

# Without colors
$ NO_COLOR=1 burrow list
✓ 3 secrets  # Same symbols, no ANSI codes
```

## Testing Output

When writing tests, set `NO_COLOR=1` to avoid ANSI escape codes in assertions:

```rust
#[test]
fn test_list_output() {
    std::env::set_var("NO_COLOR", "1");
    // ... test output ...
}
```

## Consistency Checklist

When adding new commands, ensure:

- [ ] Use `output::success()` for successful operations
- [ ] Use `output::error()` for failures (to stderr)
- [ ] Use `output::warn()` for warnings
- [ ] Use `output::hint()` for next-step suggestions
- [ ] Use `output::section()` for headers with separators
- [ ] Use `output::kv()` for labeled values
- [ ] Use `output::list_item()` for bulleted lists
- [ ] Format paths with `output::path()`
- [ ] Format commands with `output::cmd()`
- [ ] Format keys with `output::key()`
- [ ] Keep scripting commands plain (like `burrow get`)
- [ ] Test with `NO_COLOR=1`

## References

Inspired by:
- [uv](https://github.com/astral-sh/uv) - Python package manager
- [ruff](https://github.com/astral-sh/ruff) - Python linter
- [ripgrep](https://github.com/BurntSushi/ripgrep) - Fast grep alternative
- [bat](https://github.com/sharkdp/bat) - Cat clone with syntax highlighting

---

**Remember:** Consistency is key. When in doubt, look at existing commands for patterns.
