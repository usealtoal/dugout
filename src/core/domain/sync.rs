/// Result of a sync operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncResult {
    /// Number of secrets re-encrypted
    pub secrets: usize,
    /// Number of recipients in the current set
    pub recipients: usize,
    /// Whether re-encryption was actually needed
    pub was_needed: bool,
}
