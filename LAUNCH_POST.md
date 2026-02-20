# Dugout Launch Posts

## Hacker News — Show HN

**Title:** Show HN: Dugout – Git-native secrets manager with team access control, written in Rust

**Text:**

Hey HN! I built Dugout because I was tired of the secrets management tradeoff: either use a SaaS service (Vault, Doppler, Infisical) with infrastructure overhead and vendor lock-in, or use .env files that inevitably end up in git history.

Dugout stores encrypted secrets directly in your repo. No server, no SaaS, no cloud dependency. Team access is managed through git itself — teammates `knock` to request access, you `admit` them, and the re-encryption is just another git commit.

Key features:

- `dugout init` → start adding secrets in 10 seconds
- `dugout .` → auto-detects your stack (Node, Python, Rust, Go, etc.) and runs with secrets injected
- age encryption by default (~100µs per encrypt), optional AWS/GCP KMS
- Single static binary, no runtime dependencies
- Works with any git host, any CI/CD, any infrastructure

The `knock`/`admit` workflow is what I'm most excited about — it turns access control into a git-native operation. When someone needs access, they run `dugout knock`, which creates a commit with their public key. The admin runs `dugout admit <name>`, re-encrypts the vault for them, and pushes. No tokens to rotate, no dashboards to manage.

Written in Rust, ~4k lines, MIT/Apache-2.0 dual licensed.

GitHub: https://github.com/usemantle/dugout
Install: `curl -LsSf https://raw.githubusercontent.com/usemantle/dugout/main/scripts/install.sh | sh`
Homebrew: `brew install usemantle/tap/dugout`

Would love feedback on the approach and any edge cases I'm missing!

---

## Reddit — r/rust

**Title:** [Media] Dugout: Git-native secrets manager with team access control

**Text:**

Just shipped v0.1.7 of **dugout** — a secrets manager that stores encrypted values directly in your git repo. No server, no SaaS, no infrastructure.

**Why another secrets tool?**

I wanted something with zero config overhead that treats the git repo as the source of truth for everything, including secrets. The `knock`/`admit` workflow lets teammates request and receive access purely through git commits.

**Tech highlights:**

- Built with `clap` for CLI, `age` crate for encryption
- `dugout .` auto-detects your stack and spawns with secrets in env
- ~100µs encrypt, single static binary
- Optional AWS KMS / GCP KMS backends
- Comprehensive test suite with `assert_cmd`
- MIT/Apache-2.0

**The knock/admit flow:**

```
# Alice needs access
alice$ dugout knock
# Creates commit with her public key

# Bob (admin) grants access  
bob$ dugout admit alice
# Re-encrypts vault, alice can now decrypt

# No tokens, no dashboards, no rotation
```

GitHub: https://github.com/usemantle/dugout

Feedback welcome — especially on the encryption approach and any security concerns. This is early days and I want to get it right.

---

## Reddit — r/devops

**Title:** I built a secrets manager that lives entirely in your git repo — no server, no SaaS

**Text:**

I've been frustrated with secrets management tools that require either running infrastructure (Vault) or paying for SaaS (Doppler, Infisical). `.env` files are simple but insecure, and `sops` requires config files and doesn't handle team access well.

So I built **dugout** — secrets are encrypted and stored in your repo. Team access is managed through git commits. No servers, no dashboards, no vendor lock-in.

**How it works:**

1. `dugout init` — creates an encrypted vault in your repo
2. `dugout set API_KEY=sk-xxx` — encrypts and stores
3. `dugout .` — runs your app with secrets injected
4. `dugout knock` / `dugout admit` — git-native team access

**vs the alternatives:**

| | dugout | sops | dotenvx | Vault |
|---|---|---|---|---|
| No server | ✅ | ✅ | ✅ | ❌ |
| No config files | ✅ | ❌ | ✅ | ❌ |
| Team access via git | ✅ | ❌ | ❌ | ❌ |
| Auto-detect & run | ✅ | ❌ | ✅ | ❌ |
| Single binary (Rust) | ✅ | ✅ (Go) | ❌ (JS) | ✅ (Go) |

Built in Rust, MIT/Apache-2.0. Feedback welcome.

https://github.com/usemantle/dugout

---

## Launch Timing

- **Target:** Tuesday Feb 18, 2026
- **HN post time:** 8:00-9:00 AM ET (peak HN traffic)
- **Reddit posts:** Stagger 2-3 hours after HN
- **Day of week:** Tuesday-Wednesday historically best for Show HN

## Pre-Launch Checklist

- [ ] README polish — ensure install instructions are copy-paste clean
- [ ] Demo GIF/asciinema on README (already have scripts)
- [ ] Verify install.sh works fresh on macOS + Linux
- [ ] Verify homebrew formula works
- [ ] Make repo PUBLIC (currently private)
- [ ] Rob reviews and approves the posts
- [ ] Have responses ready for common questions:
  - "What if someone gains write access to the repo?"
  - "How does this compare to git-crypt?"
  - "What about key rotation?"
  - "What about CI/CD integration?"
