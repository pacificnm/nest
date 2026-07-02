//! On-disk cache entry metadata.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use nest_cache::{CacheEntry, CacheKey};
use serde::{Deserialize, Serialize};

/// JSON sidecar stored beside each cached payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileCacheMeta {
    /// Original cache key.
    pub key: String,
    /// Tags used for grouped invalidation.
    pub tags: Vec<String>,
    /// Optional unix expiry timestamp.
    pub expires_at: Option<u64>,
    /// Optional HTTP content type.
    pub content_type: Option<String>,
    /// Creation timestamp as unix seconds.
    pub created_at: u64,
}

impl FileCacheMeta {
    /// Builds metadata from a cache entry.
    pub fn from_entry(entry: &CacheEntry, content_type: Option<String>) -> Self {
        Self {
            key: entry.key.as_str().to_string(),
            tags: entry.tags.clone(),
            expires_at: entry.expires_at.and_then(system_time_to_secs),
            content_type,
            created_at: now_secs(),
        }
    }

    /// Returns whether the entry is expired at the given time.
    pub fn is_expired_at(&self, now: SystemTime) -> bool {
        self.expires_at
            .and_then(|secs| UNIX_EPOCH.checked_add(Duration::from_secs(secs)))
            .is_some_and(|expires_at| now >= expires_at)
    }

    /// Rebuilds a cache key from stored metadata.
    pub fn cache_key(&self) -> CacheKey {
        CacheKey::new(self.key.clone())
    }
}

pub(crate) fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub(crate) fn system_time_to_secs(time: SystemTime) -> Option<u64> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_expired_meta() {
        let meta = FileCacheMeta {
            key: "demo".into(),
            tags: vec![],
            expires_at: Some(1),
            content_type: None,
            created_at: 1,
        };
        let now = UNIX_EPOCH + Duration::from_secs(2);
        assert!(meta.is_expired_at(now));
    }
}
