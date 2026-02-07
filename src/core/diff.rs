//! Diff type.
//!
//! Represents the comparison between vault secrets and a local .env file.

use std::collections::{HashMap, HashSet};

/// The sync state of a single secret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryStatus {
    /// Secret exists in both vault and .env with matching values.
    Synced,
    /// Secret exists in both but values differ.
    Modified,
    /// Secret exists in vault but not in .env.
    VaultOnly,
    /// Secret exists in .env but not in vault.
    EnvOnly,
}

/// A single entry in a diff comparison.
#[derive(Debug, Clone)]
pub struct DiffEntry {
    key: String,
    status: EntryStatus,
}

impl DiffEntry {
    /// Create a new diff entry.
    pub fn new(key: String, status: EntryStatus) -> Self {
        Self { key, status }
    }

    /// The secret key name.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The sync status.
    pub fn status(&self) -> &EntryStatus {
        &self.status
    }

    /// Whether this entry is synced.
    pub fn is_synced(&self) -> bool {
        matches!(self.status, EntryStatus::Synced)
    }
}

/// The full diff between vault and .env file.
#[derive(Debug)]
pub struct Diff {
    entries: Vec<DiffEntry>,
}

impl Diff {
    /// Compute the diff between vault secrets and an env file.
    ///
    /// # Arguments
    ///
    /// * `vault_secrets` - Key-value pairs from the vault
    /// * `env_secrets` - Key-value pairs from the .env file
    ///
    /// # Returns
    ///
    /// A `Diff` containing all entries sorted by key name.
    pub fn compute(vault_secrets: &[(String, String)], env_secrets: &[(String, String)]) -> Self {
        let vault_map: HashMap<_, _> = vault_secrets.iter().cloned().collect();
        let env_map: HashMap<_, _> = env_secrets.iter().cloned().collect();

        let vault_keys: HashSet<_> = vault_map.keys().collect();
        let env_keys: HashSet<_> = env_map.keys().collect();

        let mut entries = Vec::new();

        // Find all unique keys
        let all_keys: HashSet<_> = vault_keys.union(&env_keys).collect();

        for key in all_keys {
            let vault_value = vault_map.get(*key);
            let env_value = env_map.get(*key);

            let status = match (vault_value, env_value) {
                (Some(v), Some(e)) if v == e => EntryStatus::Synced,
                (Some(_), Some(_)) => EntryStatus::Modified,
                (Some(_), None) => EntryStatus::VaultOnly,
                (None, Some(_)) => EntryStatus::EnvOnly,
                (None, None) => unreachable!("key must exist in at least one map"),
            };

            entries.push(DiffEntry::new((*key).clone(), status));
        }

        // Sort by key name for consistent output
        entries.sort_by(|a, b| a.key.cmp(&b.key));

        Self { entries }
    }

    /// All entries.
    pub fn entries(&self) -> &[DiffEntry] {
        &self.entries
    }

    /// Only synced entries.
    pub fn synced(&self) -> Vec<&DiffEntry> {
        self.entries
            .iter()
            .filter(|e| matches!(e.status, EntryStatus::Synced))
            .collect()
    }

    /// Only modified entries.
    pub fn modified(&self) -> Vec<&DiffEntry> {
        self.entries
            .iter()
            .filter(|e| matches!(e.status, EntryStatus::Modified))
            .collect()
    }

    /// Only vault-only entries.
    pub fn vault_only(&self) -> Vec<&DiffEntry> {
        self.entries
            .iter()
            .filter(|e| matches!(e.status, EntryStatus::VaultOnly))
            .collect()
    }

    /// Only env-only entries.
    pub fn env_only(&self) -> Vec<&DiffEntry> {
        self.entries
            .iter()
            .filter(|e| matches!(e.status, EntryStatus::EnvOnly))
            .collect()
    }

    /// Whether everything is in sync.
    pub fn is_synced(&self) -> bool {
        self.entries.iter().all(|e| e.is_synced())
    }

    /// Total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether there are no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_all_synced() {
        let vault = vec![
            ("API_KEY".to_string(), "secret123".to_string()),
            ("DB_URL".to_string(), "postgres://".to_string()),
        ];
        let env = vault.clone();

        let diff = Diff::compute(&vault, &env);

        assert_eq!(diff.len(), 2);
        assert!(diff.is_synced());
        assert_eq!(diff.synced().len(), 2);
        assert_eq!(diff.modified().len(), 0);
        assert_eq!(diff.vault_only().len(), 0);
        assert_eq!(diff.env_only().len(), 0);
    }

    #[test]
    fn test_diff_modified() {
        let vault = vec![("API_KEY".to_string(), "secret123".to_string())];
        let env = vec![("API_KEY".to_string(), "different".to_string())];

        let diff = Diff::compute(&vault, &env);

        assert_eq!(diff.len(), 1);
        assert!(!diff.is_synced());
        assert_eq!(diff.modified().len(), 1);
        assert_eq!(diff.modified()[0].key(), "API_KEY");
    }

    #[test]
    fn test_diff_vault_only() {
        let vault = vec![
            ("API_KEY".to_string(), "secret123".to_string()),
            ("VAULT_SECRET".to_string(), "value".to_string()),
        ];
        let env = vec![("API_KEY".to_string(), "secret123".to_string())];

        let diff = Diff::compute(&vault, &env);

        assert_eq!(diff.len(), 2);
        assert!(!diff.is_synced());
        assert_eq!(diff.vault_only().len(), 1);
        assert_eq!(diff.vault_only()[0].key(), "VAULT_SECRET");
    }

    #[test]
    fn test_diff_env_only() {
        let vault = vec![("API_KEY".to_string(), "secret123".to_string())];
        let env = vec![
            ("API_KEY".to_string(), "secret123".to_string()),
            ("UNTRACKED".to_string(), "value".to_string()),
        ];

        let diff = Diff::compute(&vault, &env);

        assert_eq!(diff.len(), 2);
        assert!(!diff.is_synced());
        assert_eq!(diff.env_only().len(), 1);
        assert_eq!(diff.env_only()[0].key(), "UNTRACKED");
    }

    #[test]
    fn test_diff_mixed() {
        let vault = vec![
            ("SYNCED".to_string(), "same".to_string()),
            ("MODIFIED".to_string(), "old".to_string()),
            ("VAULT_ONLY".to_string(), "secret".to_string()),
        ];
        let env = vec![
            ("SYNCED".to_string(), "same".to_string()),
            ("MODIFIED".to_string(), "new".to_string()),
            ("ENV_ONLY".to_string(), "local".to_string()),
        ];

        let diff = Diff::compute(&vault, &env);

        assert_eq!(diff.len(), 4);
        assert!(!diff.is_synced());
        assert_eq!(diff.synced().len(), 1);
        assert_eq!(diff.modified().len(), 1);
        assert_eq!(diff.vault_only().len(), 1);
        assert_eq!(diff.env_only().len(), 1);
    }

    #[test]
    fn test_diff_empty() {
        let vault: Vec<(String, String)> = vec![];
        let env: Vec<(String, String)> = vec![];

        let diff = Diff::compute(&vault, &env);

        assert!(diff.is_empty());
        assert!(diff.is_synced());
        assert_eq!(diff.len(), 0);
    }

    #[test]
    fn test_diff_entry_is_synced() {
        let synced = DiffEntry::new("KEY".to_string(), EntryStatus::Synced);
        let modified = DiffEntry::new("KEY".to_string(), EntryStatus::Modified);

        assert!(synced.is_synced());
        assert!(!modified.is_synced());
    }
}
