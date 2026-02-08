# Security

## Threat Model

Dugout is designed to protect secrets in the following scenarios:

✅ **Protection Against:**
- Accidental exposure of secrets in version control
- Public repository read access to encrypted `.dugout.toml` values
- Plaintext secrets stored in the vault file
- Unauthorized access by users who lack a valid team identity key
- Memory dumps revealing secrets after use (via zeroization)
- Shoulder-surfing attacks (secrets not visible in command history)

❌ **NOT Protected Against:**
- Compromised developer endpoints or CI runners where secrets are decrypted
- Compromised private keys (`~/.dugout/identity`, `~/.dugout/keys/<project>/identity.key`)
- Malicious code running with your user privileges
- Malicious insiders who already have valid decrypt keys
- Cloud IAM/KMS misconfiguration (for hybrid mode with AWS/GCP KMS)
- Physical access to an unlocked machine
- Keyloggers or memory forensics on a running system
- Side-channel attacks on encryption implementation
- Supply chain attacks on dependencies

## Encryption

### Algorithm

Dugout uses modern authenticated encryption, with **age** (Actually Good Encryption) as the default backend.

- **Age (default):** X25519 recipients + ChaCha20-Poly1305
- **Hybrid:** age + cloud KMS (AWS or GCP) — secrets encrypted for both age keys and KMS in an envelope format
- **GPG:** GPG encryption via CLI (feature-gated, `--features gpg`)

- **Cipher:** ChaCha20-Poly1305 (authenticated encryption)
- **Key Exchange:** X25519 (elliptic curve Diffie-Hellman)
- **Key Derivation:** HKDF-SHA256
- **Format:** ASCII-armored or binary age format

### What's Encrypted

- All secret values stored in `.dugout.toml`
- Secret values are never stored in plaintext inside the vault file
- Secrets are encrypted to one or more recipients (team members)
- Each secret can be decrypted by any team member with a valid private key

### What's NOT Encrypted

- Secret **names** (keys) in `.dugout.toml` are visible in plaintext
- Project metadata and configuration settings
- Team member names and public keys
- The `.dugout.toml` file structure

**Rationale:** Secret names are intentionally visible to enable:
- Diffing and merge conflict resolution in version control
- Easy discovery of available secrets without decryption
- Fast key lookups without cipher operations

If secret names are sensitive, use generic names like `API_KEY_1`, `DATABASE_URL`, etc.

## Key Storage

### Private Keys

Your private identity key is stored at one of:
```
~/.dugout/identity
~/.dugout/keys/<project>/identity.key
```

- **Permissions:** Only readable by your user (0600, enforced on Unix)
- **Format:** ASCII-armored age private key
- **Reuse:** Global identity can be reused across multiple dugout projects

**Security responsibilities:**
- Keep this file backed up securely (encrypted backup recommended)
- Never commit it to version control
- Protect your home directory with full-disk encryption
- Use a secure password manager for backup copies

### Public Keys

Team member public keys are stored in `.dugout.toml` under `[recipients]`.

- These are safe to commit to version control
- They allow encrypting secrets for multiple team members
- Removing a recipient requires re-encrypting all secrets

## Memory Handling

Dugout uses the [`zeroize`](https://crates.io/crates/zeroize) crate to securely erase decrypted secrets from memory.

### Zeroization Points

1. **After `dugout get`** - Secret is zeroized after printing to stdout
2. **After `dugout run`** - Secrets are zeroized after being passed to subprocess environment
3. **During re-encryption** - Old plaintext is zeroized after re-encrypting
4. **On error paths** - Zeroizing types ensure cleanup even on early returns

### Limitations

- Secrets passed to subprocesses remain in that process's memory
- Secrets printed to terminal may remain in scrollback buffer
- Compiler optimizations may create temporary copies
- Operating system may swap memory to disk
- Core dumps may capture memory contents

**Recommendations:**
- Use `dugout run` instead of `dugout unlock` when possible
- Clear terminal scrollback after viewing secrets
- Disable swap or use encrypted swap
- Disable core dumps for security-sensitive environments

## Attack Scenarios

### Scenario 1: Repository Compromise

**Attack:** Attacker gains read access to your Git repository.

**Protection:** Secrets are encrypted in `.dugout.toml`. Without the private key, secrets cannot be decrypted.

**Mitigation:** Rotate any secrets that were previously committed in plaintext before using dugout.

### Scenario 2: Private Key Theft

**Attack:** Attacker copies `~/.dugout/identity` (or project key files) from your machine.

**Risk:** All secrets encrypted to your public key can be decrypted.

**Mitigation:**
1. Remove your compromised key from all dugout projects
2. Generate a new identity key
3. Re-add yourself to team with new key
4. Rotate all secrets exposed by the breach

### Scenario 3: Process Memory Inspection

**Attack:** Attacker uses a debugger or memory dump tool on a running dugout process.

**Risk:** Decrypted secrets may be visible in memory during operations.

**Mitigation:**
- Dugout uses zeroization to minimize exposure window
- Run dugout in a trusted environment
- Use `dugout run` for short-lived secret access
- Consider using a secrets management service for high-security needs

### Scenario 4: `.env` File Exposure

**Attack:** `.env` file (created by `dugout unlock`) is accidentally committed.

**Risk:** All secrets are exposed in plaintext.

**Mitigation:**
- Add `.env` to `.gitignore` (dugout does this automatically)
- Use `git-secrets` or similar tools to prevent commits
- Prefer `dugout run` over `dugout unlock` to avoid creating `.env` files
- Regularly audit repository for sensitive files

## Public Repo Safety

Committing `.dugout.toml` to Git (including public repos) is an intended workflow.

✅ **Reasonably safe if you do all of the following:**
- Keep identity keys off the repo and out of CI logs/artifacts
- Never commit plaintext `.env` files
- Rotate secrets quickly after personnel or key changes
- Keep endpoint security and cloud IAM/KMS policies tight

❌ **Not enough for high-assurance / motivated-attacker environments by itself:**
- Dugout is not a full zero-trust brokered secret system
- Decrypt-capable endpoints remain the highest-risk layer

## Best Practices

### Development Workflow

1. **Use `dugout run` for development:**
   ```bash
   dugout run -- npm start
   ```
   This injects secrets without creating a `.env` file.

2. **Lock secrets after changes:**
   ```bash
   dugout set API_KEY xyz123
   git add .dugout.toml
   git commit -m "Add API_KEY"
   ```

3. **Never commit `.env` files:**
   ```bash
   echo ".env" >> .gitignore
   ```

### Team Collaboration

1. **Add team members before setting secrets:**
   ```bash
   dugout team add alice age1...
   dugout team add bob age1...
   dugout set DATABASE_URL postgres://...
   ```

2. **When removing a team member:**
   ```bash
   dugout team rm charlie
   # Secrets are automatically re-encrypted without charlie
   ```

3. **Rotate secrets when team members leave:**
   - Generate new credentials/API keys
   - Update values in dugout
   - Invalidate old credentials in the service

### CI/CD Integration

For CI/CD pipelines, consider:

1. **Option A:** Use platform secrets (GitHub Actions secrets, GitLab CI variables)
2. **Option B:** Store dugout identity in secure CI vault, fetch at runtime
3. **Option C:** Use dedicated secrets management (Vault, AWS Secrets Manager)

**Do NOT:**
- Commit CI identity keys to the repository
- Use `dugout unlock` in CI and cache `.env` files
- Share identity keys across environments

## Compliance

### Data Protection

- **GDPR:** Dugout itself does not collect or process personal data
- **Encryption at rest:** Secrets are encrypted on disk
- **Access control:** Only users with valid private keys can decrypt

### Audit Trail

Dugout does not maintain audit logs. For compliance needs:
- Use Git history to track configuration changes
- Implement application-level logging for secret access
- Use secrets management services with built-in auditing

## Reporting Security Issues

If you discover a security vulnerability in dugout:

1. **Do NOT open a public GitHub issue**
2. Email: rob@altoal.com
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Impact assessment
   - Suggested fix (if any)

We will respond within 48 hours and work with you on a coordinated disclosure.

## Further Reading

- [age encryption format](https://age-encryption.org/)
- [Zeroize crate documentation](https://docs.rs/zeroize/)
- [OWASP Secrets Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html)
