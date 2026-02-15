//! Vault
//!
//! The primary interface for all dugout operations.

use crate::core::cipher;
use crate::core::config::{self, Config};
use crate::core::constants;
use crate::core::domain::{Diff, Env, Identity, Recipient, Secret, SyncResult, VaultInfo};
use crate::core::store;
use crate::core::types::{MemberName, PublicKey, SecretKey};
use crate::error::{ConfigError, Result, SecretError, ValidationError};
use sha2::{Digest, Sha256};
use tracing::{debug, info, instrument};
use zeroize::Zeroizing;

/// The primary interface for all dugout operations
///
/// Owns the config, manages keys, and provides all secret operations.
pub struct Vault {
    config: Config,
    project_id: String,
    identity: Identity,
    backend: cipher::CipherBackend,
    vault_name: Option<String>,
}

impl std::fmt::Debug for Vault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vault")
            .field("config", &self.config)
            .field("project_id", &self.project_id)
            .field("identity", &self.identity)
            .field("backend", &self.backend)
            .field("vault_name", &self.vault_name)
            .finish()
    }
}

impl Vault {
    // --- Construction ---
    /// Open an existing vault.
    ///
    /// # Arguments
    ///
    /// * `vault` - Optional vault name (None = default `.dugout.toml`)
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::NotInitialized` if no vault config exists.
    /// Returns error if the configuration is invalid or cannot be read.
    pub fn open_vault(vault: Option<&str>) -> Result<Self> {
        let config = Config::load_from(vault)?;
        let project_id = config.project_id();

        // Identity resolution order:
        // 1. DUGOUT_IDENTITY / DUGOUT_IDENTITY_FILE env vars (CI/CD)
        // 2. Project-local identity (~/.dugout/keys/<project>/)
        // 3. Global identity (~/.dugout/identity)
        let identity = Identity::from_env()
            .filter(|id| identity_has_access(&config, id))
            .or_else(|| {
                store::load_identity(&project_id)
                    .ok()
                    .filter(|id| identity_has_access(&config, id))
            })
            .or_else(|| {
                Identity::has_global()
                    .ok()
                    .filter(|has| *has)
                    .and_then(|_| Identity::load_global().ok())
                    .filter(|id| identity_has_access(&config, id))
            })
            .ok_or(ConfigError::AccessDenied)?;

        let backend = cipher::CipherBackend::from_config(&config)?;

        Ok(Self {
            config,
            project_id,
            identity,
            backend,
            vault_name: vault.map(|s| s.to_string()),
        })
    }

    /// Open the default vault (backward compat).
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::NotInitialized` if no `.dugout.toml` exists.
    /// Returns error if the configuration is invalid or cannot be read.
    pub fn open() -> Result<Self> {
        Self::open_vault(None)
    }

    /// Initialize a new vault.
    ///
    /// # Arguments
    ///
    /// * `vault` - Optional vault name (None = default `.dugout.toml`)
    /// * `name` - Name of the first recipient (the initializing user)
    /// * `kms_key` - Optional KMS key for hybrid encryption
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::AlreadyInitialized` if vault already exists.
    /// Returns error if keypair generation or file operations fail.
    pub fn init_vault(vault: Option<&str>, name: &str, kms_key: Option<String>) -> Result<Self> {
        validate_member_name(name)?;

        if Config::exists_for(vault) {
            return Err(ConfigError::AlreadyInitialized.into());
        }

        let mut config = Config::new();

        // Enable hybrid mode if KMS key provided
        if let Some(ref key) = kms_key {
            config.kms = Some(crate::core::config::KmsConfig { key: key.clone() });
        }

        let project_id = config.project_id();

        // Priority for identity:
        // 1. Existing project key (for multi-vault in same directory)
        // 2. Global identity (copy to project dir)
        // 3. Generate new project key
        let (public_key, identity) = if store::has_key(&project_id) {
            // Reuse existing project key (multi-vault scenario)
            let id = store::load_identity(&project_id)?;
            let pk = id.public_key();
            (pk, id)
        } else if Identity::has_global()? {
            let global_pubkey = Identity::load_global_pubkey()?;
            let global_identity = Identity::load_global()?;
            // Copy global key into project key dir so open() can find it
            let key_dir = Identity::project_dir(&project_id)?;
            std::fs::create_dir_all(&key_dir)?;
            let project_key_path = key_dir.join("identity.key");
            if !project_key_path.exists() {
                std::fs::copy(Identity::global_path()?, &project_key_path)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(
                        &project_key_path,
                        std::fs::Permissions::from_mode(0o600),
                    )?;
                }
            }
            (global_pubkey, global_identity)
        } else {
            let pk = store::generate_keypair(&project_id)?;
            let id = store::load_identity(&project_id)?;
            (pk, id)
        };

        config
            .recipients
            .insert(name.to_string(), public_key.clone());
        config.save_to(vault)?;

        config::ensure_gitignore()?;
        let backend = cipher::CipherBackend::from_config(&config)?;

        Ok(Self {
            config,
            project_id,
            identity,
            backend,
            vault_name: vault.map(|s| s.to_string()),
        })
    }

    /// Initialize the default vault (backward compat).
    ///
    /// Creates a new `.dugout.toml` configuration file, generates a keypair,
    /// and adds the specified user as the first recipient.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::AlreadyInitialized` if vault already exists.
    /// Returns error if keypair generation or file operations fail.
    pub fn init(name: &str, kms_key: Option<String>) -> Result<Self> {
        Self::init_vault(None, name, kms_key)
    }

    /// Config reference
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Vault's identity
    pub fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Project ID derived from directory name
    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    // --- Secrets ---
    /// Set a secret, encrypting for all configured recipients
    ///
    /// # Errors
    ///
    /// Returns `ValidationError` if key or value is invalid.
    /// Returns `SecretError::AlreadyExists` if key exists and `force` is false.
    #[instrument(skip(self, value))]
    pub fn set(&mut self, key: &str, value: &str, force: bool) -> Result<Secret> {
        info!(key = %key, force = force, "setting secret");

        // Validate input
        validate_key(key)?;
        validate_value(key, value)?;

        if self.config.secrets.contains_key(key) && !force {
            return Err(SecretError::AlreadyExists(key.to_string()).into());
        }

        let recipients = get_recipients_as_strings(&self.config);
        if recipients.is_empty() {
            return Err(ConfigError::NoRecipients.into());
        }

        let encrypted = self.backend.encrypt(value, &recipients)?;

        self.config
            .secrets
            .insert(key.to_string(), encrypted.clone());
        self.update_recipients_hash();
        self.config.save_to(self.vault_name.as_deref())?;

        debug!(key = %key, "secret set, saving config");
        Ok(Secret::new(key.to_string(), encrypted))
    }

    /// Get a decrypted secret
    ///
    /// Returns the decrypted plaintext value wrapped in `Zeroizing` for secure memory cleanup.
    ///
    /// # Errors
    ///
    /// Returns `SecretError::NotFound` if the key doesn't exist.
    /// Returns `CipherError` if decryption fails.
    #[instrument(skip(self))]
    pub fn get(&self, key: &str) -> Result<Zeroizing<String>> {
        let encrypted = self.config.secrets.get(key).ok_or_else(|| {
            let available: Vec<String> = self.config.secrets.keys().cloned().collect();
            SecretError::not_found_with_suggestions(key.to_string(), &available)
        })?;

        let plaintext = self.backend.decrypt(encrypted, self.identity.as_age())?;

        Ok(Zeroizing::new(plaintext))
    }

    /// Remove a secret.
    ///
    /// # Arguments
    ///
    /// * `key` - Secret key name
    ///
    /// # Errors
    ///
    /// Returns `SecretError::NotFound` if the key doesn't exist.
    #[instrument(skip(self))]
    pub fn remove(&mut self, key: &str) -> Result<()> {
        info!(key = %key, "removing secret");

        if self.config.secrets.remove(key).is_none() {
            let available: Vec<String> = self.config.secrets.keys().cloned().collect();
            return Err(
                SecretError::not_found_with_suggestions(key.to_string(), &available).into(),
            );
        }
        self.config.save_to(self.vault_name.as_deref())?;
        Ok(())
    }

    /// List all secrets.
    pub fn list(&self) -> Vec<Secret> {
        self.config
            .secrets
            .iter()
            .map(|(key, value)| Secret::new(key.clone(), value.clone()))
            .collect()
    }

    /// Decrypt all secrets
    ///
    /// Returns vector of (key, plaintext_value) pairs with values in `Zeroizing` for secure cleanup.
    ///
    /// # Errors
    ///
    /// Returns error if decryption of any secret fails.
    #[instrument(skip(self))]
    pub fn decrypt_all(&self) -> Result<Vec<(SecretKey, Zeroizing<String>)>> {
        debug!(count = self.config.secrets.len(), "decrypting all secrets");

        let mut pairs = Vec::new();
        for (key, encrypted) in &self.config.secrets {
            let plaintext = self.backend.decrypt(encrypted, self.identity.as_age())?;
            pairs.push((key.clone(), Zeroizing::new(plaintext)));
        }

        Ok(pairs)
    }

    /// Re-encrypt all secrets for the current recipient set
    ///
    /// Call this after adding or removing team members.
    ///
    /// # Errors
    ///
    /// Returns error if decryption or re-encryption fails.
    pub fn reencrypt_all(&mut self) -> Result<()> {
        let recipients = get_recipients_as_strings(&self.config);

        let mut updated = std::collections::BTreeMap::new();
        for (key, encrypted) in &self.config.secrets {
            // Use Zeroizing to ensure plaintext is wiped after re-encryption
            let plaintext =
                Zeroizing::new(self.backend.decrypt(encrypted, self.identity.as_age())?);
            let reencrypted = self.backend.encrypt(&plaintext, &recipients)?;
            updated.insert(key.clone(), reencrypted);
        }

        self.config.secrets = updated;
        self.update_recipients_hash();
        self.config.save_to(self.vault_name.as_deref())?;

        Ok(())
    }

    // --- Team ---
    /// Add a team member and re-encrypt all secrets for them
    ///
    /// # Errors
    ///
    /// Returns `CipherError` if the public key is invalid.
    /// Returns error if re-encryption fails.
    #[instrument(skip(self, key))]
    pub fn add_recipient(&mut self, name: &str, key: &str) -> Result<()> {
        info!(name = %name, "adding team member");

        validate_member_name(name)?;

        // Validate the key format first - this will return a clear error if invalid
        cipher::parse_recipient(key)?;

        self.config
            .recipients
            .insert(name.to_string(), key.to_string());
        self.config.save_to(self.vault_name.as_deref())?;

        // Re-encrypt all secrets for the new recipient set
        if !self.config.secrets.is_empty() {
            self.reencrypt_all()?;
        }

        Ok(())
    }

    /// Remove a team member and re-encrypt all secrets without them
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::RecipientNotFound` if the member doesn't exist.
    /// Returns error if re-encryption fails.
    #[instrument(skip(self))]
    pub fn remove_recipient(&mut self, name: &str) -> Result<()> {
        info!(name = %name, "removing team member");

        if self.config.recipients.remove(name).is_none() {
            return Err(ConfigError::RecipientNotFound(name.to_string()).into());
        }
        self.config.save_to(self.vault_name.as_deref())?;

        // Re-encrypt all secrets without the removed recipient
        if !self.config.secrets.is_empty() {
            self.reencrypt_all()?;
        }

        Ok(())
    }

    /// List all team members
    pub fn recipients(&self) -> Vec<Recipient> {
        list_recipients(&self.config)
            .into_iter()
            .filter_map(|(name, key)| Recipient::new(name, key).ok())
            .collect()
    }

    /// Migrate legacy request files to per-vault directories.
    ///
    /// Moves files from `.dugout/requests/*.pub` to `.dugout/requests/default/*.pub`.
    /// This is called automatically when listing pending requests.
    fn migrate_legacy_requests() -> Result<()> {
        let legacy_dir = std::path::Path::new(".dugout/requests");
        let new_dir = constants::request_dir(None); // .dugout/requests/default

        if !legacy_dir.exists() {
            return Ok(());
        }

        // Check for .pub files directly in .dugout/requests/ (legacy location)
        let mut has_legacy_files = false;
        for entry in std::fs::read_dir(legacy_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("pub") {
                has_legacy_files = true;
                break;
            }
        }

        if !has_legacy_files {
            return Ok(());
        }

        // Create new directory if needed
        std::fs::create_dir_all(&new_dir)?;

        // Move files
        for entry in std::fs::read_dir(legacy_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("pub") {
                if let Some(filename) = path.file_name() {
                    let new_path = new_dir.join(filename);
                    if !new_path.exists() {
                        std::fs::rename(&path, &new_path)?;
                        debug!(from = %path.display(), to = %new_path.display(), "migrated legacy request file");
                    }
                }
            }
        }

        Ok(())
    }

    /// List pending access requests
    ///
    /// Returns a vector of (name, public_key) pairs from `.dugout/requests/` directory.
    ///
    /// # Errors
    ///
    /// Returns error if the directory cannot be read.
    pub fn pending_requests(&self) -> Result<Vec<(String, String)>> {
        // Migrate legacy requests on first access
        Self::migrate_legacy_requests()?;

        let requests_dir = constants::request_dir(self.vault_name.as_deref());

        if !requests_dir.exists() {
            return Ok(Vec::new());
        }

        let mut requests = Vec::new();
        for entry in std::fs::read_dir(requests_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("pub") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let pubkey = std::fs::read_to_string(&path)?.trim().to_string();

                requests.push((name, pubkey));
            }
        }

        Ok(requests)
    }

    /// Admit a team member from a pending request
    ///
    /// Reads the request file, adds the recipient, deletes the request file,
    /// and re-encrypts all secrets for the new team.
    ///
    /// # Errors
    ///
    /// Returns error if the request file doesn't exist or operations fail.
    #[instrument(skip(self))]
    pub fn admit(&mut self, name: &str) -> Result<()> {
        info!(name = %name, "admitting team member from request");

        validate_member_name(name)?;

        let request_path =
            constants::request_dir(self.vault_name.as_deref()).join(format!("{}.pub", name));

        if !request_path.exists() {
            return Err(ConfigError::RecipientNotFound(format!(
                "no pending request from '{}'",
                name
            ))
            .into());
        }

        let pubkey = std::fs::read_to_string(&request_path)?.trim().to_string();

        // Add the recipient
        self.add_recipient(name, &pubkey)?;

        // Delete the request file
        std::fs::remove_file(&request_path)?;

        debug!(name = %name, "team member admitted, request file deleted");
        Ok(())
    }

    // --- Lifecycle ---
    /// Import secrets from .env file
    ///
    /// Reads key=value pairs from the file and encrypts them.
    /// Returns vector of imported secret keys.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or secrets cannot be encrypted.
    #[instrument(skip(self, path))]
    pub fn import(&mut self, path: impl AsRef<std::path::Path>) -> Result<Vec<SecretKey>> {
        let path_str = path.as_ref().display().to_string();
        info!(path = %path_str, "importing secrets");

        let env = Env::load(path)?;
        let mut imported = Vec::new();

        for (key, value) in env.entries() {
            // Validate input
            validate_key(key)?;
            validate_value(key, value)?;

            let recipients = get_recipients_as_strings(&self.config);
            if recipients.is_empty() {
                return Err(ConfigError::NoRecipients.into());
            }

            let encrypted = self.backend.encrypt(value, &recipients)?;
            self.config.secrets.insert(key.clone(), encrypted);
            imported.push(key.clone());
        }

        self.update_recipients_hash();
        self.config.save_to(self.vault_name.as_deref())?;
        debug!(count = imported.len(), "import complete");
        Ok(imported)
    }

    /// Export all decrypted secrets as .env format
    ///
    /// # Errors
    ///
    /// Returns error if decryption fails.
    #[instrument(skip(self))]
    pub fn export(&self) -> Result<Env> {
        info!("exporting secrets as env");

        let pairs = self
            .decrypt_all()?
            .into_iter()
            .map(|(k, v)| (k, v.to_string()))
            .collect();

        Ok(Env::from_pairs(pairs, std::path::PathBuf::from(".env")))
    }

    /// Unlock to .env file.
    ///
    /// Decrypts all secrets and writes them to `.env` in the current directory.
    ///
    /// # Returns
    ///
    /// The written `Env` file.
    ///
    /// # Errors
    ///
    /// Returns error if decryption or file write fails.
    #[instrument(skip(self))]
    pub fn unlock(&self) -> Result<Env> {
        info!("unlocking vault to .env");

        let env = self.export()?;
        env.save()?;

        debug!(count = env.len(), "unlock complete");
        Ok(env)
    }

    /// Compute diff between vault and .env file.
    ///
    /// Compares the vault's secrets with a .env file.
    ///
    /// # Arguments
    ///
    /// * `env_path` - Path to the .env file (defaults to `.env`)
    ///
    /// # Returns
    ///
    /// A `Diff` showing the comparison.
    ///
    /// # Errors
    ///
    /// Returns error if decryption fails or .env file cannot be read.
    pub fn diff(&self, env_path: impl AsRef<std::path::Path>) -> Result<Diff> {
        let vault_pairs = self
            .decrypt_all()?
            .into_iter()
            .map(|(k, v)| (k, v.to_string()))
            .collect::<Vec<_>>();

        let env_pairs = if env_path.as_ref().exists() {
            let env = Env::load(env_path)?;
            env.entries().to_vec()
        } else {
            Vec::new()
        };

        Ok(Diff::compute(&vault_pairs, &env_pairs))
    }

    // --- Sync ---

    /// Compute SHA-256 fingerprint of the current recipient set.
    ///
    /// Deterministic: sorts keys before hashing so order doesn't matter.
    pub fn recipients_fingerprint(&self) -> String {
        recipients_fingerprint(&self.config)
    }

    /// Check if secrets need to be re-encrypted for the current recipient set.
    ///
    /// Compares stored `recipients_hash` against the current fingerprint.
    /// Returns `true` if hash is missing (backward compat) or mismatched.
    /// Returns `false` if there are no secrets (nothing to sync).
    pub fn needs_sync(&self) -> bool {
        if self.config.secrets.is_empty() {
            return false;
        }
        match &self.config.dugout.recipients_hash {
            Some(stored) => stored != &self.recipients_fingerprint(),
            None => true, // no hash stored → assume out of sync
        }
    }

    /// Sync all secrets for the current recipient set.
    ///
    /// Re-encrypts if the recipient fingerprint has changed (or if `force` is true).
    /// Updates the stored fingerprint after re-encryption.
    ///
    /// # Errors
    ///
    /// Returns error if decryption or re-encryption fails.
    #[instrument(skip(self))]
    pub fn sync(&mut self, force: bool) -> Result<SyncResult> {
        let secrets = self.config.secrets.len();
        let recipients = self.config.recipients.len();
        let needed = force || self.needs_sync();

        if !needed {
            debug!("already in sync, skipping re-encryption");
            return Ok(SyncResult {
                secrets,
                recipients,
                was_needed: false,
            });
        }

        if secrets > 0 {
            info!(
                secrets,
                recipients, "syncing secrets for current recipients"
            );
            self.reencrypt_all()?;
        }

        // Update fingerprint (reencrypt_all already saved, but we need the hash)
        self.config.dugout.recipients_hash = Some(self.recipients_fingerprint());
        self.config.save_to(self.vault_name.as_deref())?;

        Ok(SyncResult {
            secrets,
            recipients,
            was_needed: true,
        })
    }

    /// Get the vault name (None = default vault)
    pub fn vault_name(&self) -> Option<&str> {
        self.vault_name.as_deref()
    }

    /// Update the recipients hash in config.
    ///
    /// Call this after any operation that writes secrets.
    fn update_recipients_hash(&mut self) {
        self.config.dugout.recipients_hash = Some(self.recipients_fingerprint());
    }

    /// Find all vault files in the current directory.
    ///
    /// Returns paths to all `.dugout*.toml` files.
    pub fn find_vault_files() -> Result<Vec<std::path::PathBuf>> {
        let mut vaults = Vec::new();

        for entry in std::fs::read_dir(".")? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name == ".dugout.toml"
                    || (name.starts_with(".dugout.") && name.ends_with(".toml"))
                {
                    vaults.push(path);
                }
            }
        }

        // Sort for consistent ordering
        vaults.sort();
        Ok(vaults)
    }

    /// List all vaults with their info.
    ///
    /// Returns info about each vault including access status.
    pub fn list_vaults() -> Result<Vec<VaultInfo>> {
        use crate::core::constants::vault_name_from_path;

        let vault_files = Self::find_vault_files()?;
        let mut infos = Vec::new();

        // Try to get current identity for access check
        let identity_pubkey = Identity::load_global_pubkey().ok();

        for path in vault_files {
            let vault_name = vault_name_from_path(&path);

            // Load config without identity check (we just want metadata)
            let config_path = &path;
            if !config_path.exists() {
                continue;
            }

            let contents = std::fs::read_to_string(config_path)?;
            let config: Config = toml::from_str(&contents).map_err(ConfigError::Parse)?;

            let has_access = identity_pubkey
                .as_ref()
                .map(|pk| config.recipients.values().any(|k| k == pk))
                .unwrap_or(false);

            infos.push(VaultInfo {
                name: vault_name.unwrap_or_else(|| "default".to_string()),
                path: path.clone(),
                secret_count: config.secrets.len(),
                recipient_count: config.recipients.len(),
                has_access,
            });
        }

        Ok(infos)
    }

    /// Check if multiple vaults exist.
    pub fn has_multiple_vaults() -> Result<bool> {
        Ok(Self::find_vault_files()?.len() > 1)
    }
}

// --- Private helpers ---

/// Validate a secret key name
///
/// Secret keys must be valid environment variable names:
/// - Only A-Z, 0-9, and underscore
/// - Cannot start with a digit
/// - Cannot be empty
pub(crate) fn validate_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(ValidationError::EmptyKey.into());
    }

    if let Some(first_char) = key.chars().next() {
        if first_char.is_ascii_digit() {
            return Err(ValidationError::InvalidKey {
                key: key.to_string(),
                reason: "cannot start with a digit".to_string(),
            }
            .into());
        }
    }

    for (i, ch) in key.chars().enumerate() {
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            return Err(ValidationError::InvalidKey {
                key: key.to_string(),
                reason: format!(
                    "invalid character '{}' at position {}. Only A-Z, 0-9, and underscore are allowed",
                    ch, i + 1
                ),
            }
            .into());
        }
    }

    Ok(())
}

/// Validate a team member name.
///
/// Names are used in user-facing output and request-file paths, so they must
/// stay path-safe and shell-friendly.
pub(crate) fn validate_member_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(ValidationError::InvalidMemberName {
            name: name.to_string(),
            reason: "cannot be empty".to_string(),
        }
        .into());
    }

    if name.len() > 64 {
        return Err(ValidationError::InvalidMemberName {
            name: name.to_string(),
            reason: "must be at most 64 characters".to_string(),
        }
        .into());
    }

    if name.starts_with('.') {
        return Err(ValidationError::InvalidMemberName {
            name: name.to_string(),
            reason: "cannot start with '.'".to_string(),
        }
        .into());
    }

    for (i, ch) in name.chars().enumerate() {
        if !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-' && ch != '.' && ch != '@' {
            return Err(ValidationError::InvalidMemberName {
                name: name.to_string(),
                reason: format!(
                    "invalid character '{}' at position {}. Allowed: A-Z, a-z, 0-9, _, -, ., @",
                    ch,
                    i + 1
                ),
            }
            .into());
        }
    }

    Ok(())
}

/// Validate a secret value (cannot be empty)
fn validate_value(key: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(ValidationError::EmptyValue(key.to_string()).into());
    }

    Ok(())
}

/// Get all recipient public keys as strings
fn get_recipients_as_strings(config: &Config) -> Vec<String> {
    config.recipients.values().cloned().collect()
}

/// Compute SHA-256 fingerprint of sorted recipient public keys.
fn recipients_fingerprint(config: &Config) -> String {
    let mut keys: Vec<&str> = config.recipients.values().map(|k| k.as_str()).collect();
    keys.sort();
    let joined = keys.join("\n");
    let hash = Sha256::digest(joined.as_bytes());
    format!("{:x}", hash)
}

/// List all team members as (name, public_key) pairs
fn list_recipients(config: &Config) -> Vec<(MemberName, PublicKey)> {
    config
        .recipients
        .iter()
        .map(|(name, key)| (name.clone(), key.clone()))
        .collect()
}

fn identity_has_access(config: &Config, identity: &Identity) -> bool {
    let identity_pubkey = identity.public_key();
    config
        .recipients
        .values()
        .any(|key| key == &identity_pubkey)
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    struct TestContext {
        _tmp: TempDir,
        _original_dir: std::path::PathBuf,
    }

    impl Drop for TestContext {
        fn drop(&mut self) {
            // Restore original directory before tempdir is cleaned up
            let _ = std::env::set_current_dir(&self._original_dir);
        }
    }

    fn setup_test_vault() -> (TestContext, Vault) {
        let tmp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        let vault = Vault::init("alice", None).unwrap();
        let ctx = TestContext {
            _tmp: tmp,
            _original_dir: original_dir,
        };
        (ctx, vault)
    }

    // --- Secrets tests ---

    #[test]
    fn test_vault_set_and_get() {
        let (_ctx, mut vault) = setup_test_vault();

        vault.set("API_KEY", "secret123", false).unwrap();
        let value = vault.get("API_KEY").unwrap();

        assert_eq!(value.as_str(), "secret123");
    }

    #[test]
    fn test_vault_remove() {
        let (_ctx, mut vault) = setup_test_vault();

        vault.set("TEMP_SECRET", "value", false).unwrap();
        vault.remove("TEMP_SECRET").unwrap();

        // After removal, get should fail
        assert!(vault.get("TEMP_SECRET").is_err());

        // Verify it's not in the list
        let secrets = vault.list();
        let keys: Vec<String> = secrets.iter().map(|s| s.key().to_string()).collect();
        assert!(!keys.contains(&"TEMP_SECRET".to_string()));
    }

    #[test]
    fn test_vault_list() {
        let (_ctx, mut vault) = setup_test_vault();

        vault.set("KEY_ONE", "value1", false).unwrap();
        vault.set("KEY_TWO", "value2", false).unwrap();
        vault.set("KEY_THREE", "value3", false).unwrap();

        let secrets = vault.list();
        assert_eq!(secrets.len(), 3);

        let keys: Vec<String> = secrets.iter().map(|s| s.key().to_string()).collect();
        assert!(keys.contains(&"KEY_ONE".to_string()));
        assert!(keys.contains(&"KEY_TWO".to_string()));
        assert!(keys.contains(&"KEY_THREE".to_string()));
    }

    // --- Team tests ---

    #[test]
    fn test_vault_add_recipient() {
        let (_ctx, mut vault) = setup_test_vault();

        // Set a secret first
        vault.set("SHARED_SECRET", "value", false).unwrap();

        // Generate a second keypair
        let identity = age::x25519::Identity::generate();
        let pubkey = identity.to_public().to_string();

        // Add the new recipient
        vault.add_recipient("bob", &pubkey).unwrap();

        // Verify the recipient was added
        let recipients = vault.recipients();
        assert_eq!(recipients.len(), 2);
        assert!(recipients.iter().any(|r| r.name() == "bob"));

        // Verify the secret can still be decrypted (by alice's key)
        let value = vault.get("SHARED_SECRET").unwrap();
        assert_eq!(value.as_str(), "value");
    }

    #[test]
    fn test_vault_remove_recipient() {
        let (_ctx, mut vault) = setup_test_vault();

        // Add a second recipient
        let identity = age::x25519::Identity::generate();
        let pubkey = identity.to_public().to_string();
        vault.add_recipient("bob", &pubkey).unwrap();

        assert_eq!(vault.recipients().len(), 2);

        // Remove bob
        vault.remove_recipient("bob").unwrap();

        let recipients = vault.recipients();
        assert_eq!(recipients.len(), 1);
        assert!(recipients.iter().all(|r| r.name() != "bob"));
    }

    #[test]
    fn test_vault_reencrypt_after_team_change() {
        let (_ctx, mut vault) = setup_test_vault();

        // Set a secret
        vault.set("TEAM_SECRET", "original", false).unwrap();

        // Add a new member
        let identity = age::x25519::Identity::generate();
        let pubkey = identity.to_public().to_string();
        vault.add_recipient("bob", &pubkey).unwrap();

        // Secret should still decrypt to the same value (using alice's key)
        let value = vault.get("TEAM_SECRET").unwrap();
        assert_eq!(value.as_str(), "original");

        // Verify re-encryption worked - decrypt all should succeed
        let all_secrets = vault.decrypt_all().unwrap();
        assert_eq!(all_secrets.len(), 1);
        assert_eq!(all_secrets[0].0, "TEAM_SECRET");
        assert_eq!(all_secrets[0].1.as_str(), "original");
    }

    #[test]
    fn test_vault_open_denies_non_member_identity() {
        let (_ctx, _vault) = setup_test_vault();

        let outsider = age::x25519::Identity::generate();
        let outsider_key = outsider.to_public().to_string();

        let mut cfg = Config::load().unwrap();
        cfg.recipients.clear();
        cfg.recipients.insert("bob".to_string(), outsider_key);
        cfg.save().unwrap();

        let err = Vault::open().unwrap_err();
        assert!(matches!(
            err,
            crate::error::Error::Config(ConfigError::AccessDenied)
        ));
    }

    #[test]
    fn test_validate_member_name_rejects_path_separators() {
        let result = validate_member_name("../bob");
        assert!(result.is_err());
    }

    // --- Lifecycle tests ---

    #[test]
    fn test_vault_import() {
        let (_ctx, mut vault) = setup_test_vault();

        let env_content = "IMPORT_ONE=value1\nIMPORT_TWO=value2\n";
        fs::write(".env.test", env_content).unwrap();

        let imported = vault.import(".env.test").unwrap();
        assert_eq!(imported.len(), 2);

        assert_eq!(vault.get("IMPORT_ONE").unwrap().as_str(), "value1");
        assert_eq!(vault.get("IMPORT_TWO").unwrap().as_str(), "value2");
    }

    #[test]
    fn test_vault_export_roundtrip() {
        let (_ctx, mut vault) = setup_test_vault();

        vault.set("EXPORT_KEY", "export_value", false).unwrap();
        vault.set("ANOTHER_KEY", "another_value", false).unwrap();

        let env = vault.export().unwrap();
        let exported = format!("{}", env);

        assert!(exported.contains("EXPORT_KEY=export_value"));
        assert!(exported.contains("ANOTHER_KEY=another_value"));
    }
}
