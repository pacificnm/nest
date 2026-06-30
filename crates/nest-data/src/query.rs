//! Minimal list query contract (v1).

/// Simple pagination for list operations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ListQuery {
    /// Maximum rows to return.
    pub limit: Option<u64>,
    /// Rows to skip before returning results.
    pub offset: Option<u64>,
}

impl ListQuery {
    /// Creates an empty list query (no limit/offset).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the limit.
    pub fn with_limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the offset.
    pub fn with_offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }
}
