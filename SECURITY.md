# Security

## Threat Model

Burrow is designed to protect secrets in the following scenarios:

✅ **Protection Against:**
- Accidental exposure of secrets in version control
- Plaintext secrets stored on disk
- Unauthorized access by users who lack the private key
- Memory dumps revealing secrets after use (via zeroization)
- Shoulder-surfing attacks (secrets not visible in command history)

❌ **NOT Protected Against:**
- Compromised private keys (`~/.config/burrow/identity`)
- Malicious code running with your user privileges
- Physical access to an unlocked machine
- Keyloggers or memory forensics on a running system
- Side-channel attacks on encryption implementation
- Supply chain attacks on dependencies

## Encryption

### Algorithm

Burrow uses **age** (Actually Good Encryption) with X25519 key pairs for encrypting secrets.

- **Cipher:** ChaCha20-Poly1305 (authenticated encryption)
- **Key Exchange:** X25519 (elliptic curve Diffie-Hellman)
- **Key Derivation:** HKDF-SHA256
- **Format:** ASCII-armored or binary age format

### What's Encrypted

- All secret values stored in `.burrow.toml`
- Secrets are encrypted to one or more recipients (team members)
- Each secret can be decrypted by any team member with a valid private key

### What's NOT Encrypted

- Secret **names** (keys) in `.burrow.toml` are visible in plaintext
- Project metadata and configuration settings
- Team member names and public keys
- The `.burrow.toml` file structure

**Rationale:** Secret names are intentionally visible to enable:
- Diffing and merge conflict resolution in version control
- Easy discovery of available secrets without decryption
- Fast key lookups without cipher operations

If secret names are sensitive, use generic names like `API_KEY_1`, `DATABASE_URL`, etc.

## Key Storage

### Private Keys

Your private identity key is stored at:
```
~/.config/burrow/identity
```

- **Permissions:** Only readable by your user (0600)
- **Format:** ASCII-armored age private key
- **Reuse:** Same key can be used across multiple burrow projects

**Security responsibilities:**
- Keep this file backed up securely (encrypted backup recommended)
- Never commit it to version control
- Protect your home directory with full-disk encryption
- Use a secure password manager for backup copies

### Public Keys

Team member public keys are stored in `.burrow.toml` under `[recipients]`.

- These are safe to commit to version control
- They allow encrypting secrets for multiple team members
- Removing a recipient requires re-encrypting all secrets

## Memory Handling

Burrow uses the [`zeroize`](https://crates.io/crates/zeroize) crate to securely erase decrypted secrets from memory.

### Zeroization Points

1. **After `burrow get`** - Secret is zeroized after printing to stdout
2. **After `burrow run`** - Secrets are zeroized after being passed to subprocess environment
3. **During re-encryption** - Old plaintext is zeroized after re-encrypting
4. **On error paths** - Zeroizing types ensure cleanup even on early returns

### Limitations

- Secrets passed to subprocesses remain in that process's memory
- Secrets printed to terminal may remain in scrollback buffer
- Compiler optimizations may create temporary copies
- Operating system may swap memory to disk
- Core dumps may capture memory contents

**Recommendations:**
- Use `burrow run` instead of `burrow unlock` when possible
- Clear terminal scrollback after viewing secrets
- Disable swap or use encrypted swap
- Disable core dumps for security-sensitive environments

## Attack Scenarios

### Scenario 1: Repository Compromise

**Attack:** Attacker gains read access to your Git repository.

**Protection:** Secrets are encrypted in `.burrow.toml`. Without the private key, secrets cannot be decrypted.

**Mitigation:** Rotate any secrets that were previously committed in plaintext before using burrow.

### Scenario 2: Private Key Theft

**Attack:** Attacker copies `~/.config/burrow/identity` from your machine.

**Risk:** All secrets encrypted to your public key can be decrypted.

**Mitigation:**
1. Remove your compromised key from all burrow projects
2. Generate a new identity key
3. Re-add yourself to team with new key
4. Rotate all secrets exposed by the breach

### Scenario 3: Process Memory Inspection

**Attack:** Attacker uses a debugger or memory dump tool on a running burrow process.

**Risk:** Decrypted secrets may be visible in memory during operations.

**Mitigation:**
- Burrow uses zeroization to minimize exposure window
- Run burrow in a trusted environment
- Use `burrow run` for short-lived secret access
- Consider using a secrets management service for high-security needs

### Scenario 4: `.env` File Exposure

**Attack:** `.env` file (created by `burrow unlock`) is accidentally committed.

**Risk:** All secrets are exposed in plaintext.

**Mitigation:**
- Add `.env` to `.gitignore` (burrow does this automatically)
- Use `git-secrets` or similar tools to prevent commits
- Prefer `burrow run` over `burrow unlock` to avoid creating `.env` files
- Regularly audit repository for sensitive files

## Best Practices

### Development Workflow

1. **Use `burrow run` for development:**
   ```bash
   burrow run -- npm start
   ```
   This injects secrets without creating a `.env` file.

2. **Lock secrets after changes:**
   ```bash
   burrow set API_KEY xyz123
   git add .burrow.toml
   git commit -m "Add API_KEY"
   ```

3. **Never commit `.env` files:**
   ```bash
   echo ".env" >> .gitignore
   ```

### Team Collaboration

1. **Add team members before setting secrets:**
   ```bash
   burrow team add alice age1...
   burrow team add bob age1...
   burrow set DATABASE_URL postgres://...
   ```

2. **When removing a team member:**
   ```bash
   burrow team rm charlie
   # Secrets are automatically re-encrypted without charlie
   ```

3. **Rotate secrets when team members leave:**
   - Generate new credentials/API keys
   - Update values in burrow
   - Invalidate old credentials in the service

### CI/CD Integration

For CI/CD pipelines, consider:

1. **Option A:** Use platform secrets (GitHub Actions secrets, GitLab CI variables)
2. **Option B:** Store burrow identity in secure CI vault, fetch at runtime
3. **Option C:** Use dedicated secrets management (Vault, AWS Secrets Manager)

**Do NOT:**
- Commit CI identity keys to the repository
- Use `burrow unlock` in CI and cache `.env` files
- Share identity keys across environments

## Compliance

### Data Protection

- **GDPR:** Burrow itself does not collect or process personal data
- **Encryption at rest:** Secrets are encrypted on disk
- **Access control:** Only users with valid private keys can decrypt

### Audit Trail

Burrow does not maintain audit logs. For compliance needs:
- Use Git history to track configuration changes
- Implement application-level logging for secret access
- Use secrets management services with built-in auditing

## Reporting Security Issues

If you discover a security vulnerability in burrow:

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
