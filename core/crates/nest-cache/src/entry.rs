//! Cache entry model.

use std::time::SystemTime;

use crate::key::CacheKey;

/// One cached payload and its metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry {
    /// Entry key.
    pub key: CacheKey,
    /// Raw payload bytes.
    pub value: Vec<u8>,
    /// Tags used for grouped invalidation.
    pub tags: Vec<String>,
    /// Optional expiry time.
    pub expires_at: Option<SystemTime>,
}

impl CacheEntry {
    /// Returns whether the entry is expired at the given time.
    pub fn is_expired_at(&self, now: SystemTime) -> bool {
        self.expires_at.is_some_and(|expires_at| now >= expires_at)
    }
}
