//! Cache adapter trait and in-memory implementation.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::SystemTime;

use crate::entry::CacheEntry;
use crate::error::{CacheError, CacheResult};
use crate::key::CacheKey;

/// Storage backend for [`crate::Cache`].
pub trait CacheAdapter: Send + Sync {
    /// Returns cached bytes when present and not expired.
    fn get(&self, key: &CacheKey) -> CacheResult<Option<Vec<u8>>>;

    /// Stores or replaces an entry.
    fn set(&self, entry: CacheEntry) -> CacheResult<()>;

    /// Deletes one entry.
    fn delete(&self, key: &CacheKey) -> CacheResult<()>;

    /// Deletes every entry tagged with `tag`.
    fn invalidate_tag(&self, tag: &str) -> CacheResult<u64>;

    /// Deletes every entry.
    fn clear(&self) -> CacheResult<()>;
}

/// In-process cache backed by a `HashMap`.
#[derive(Debug, Default)]
pub struct MemoryCacheAdapter {
    entries: Mutex<HashMap<CacheKey, CacheEntry>>,
}

impl MemoryCacheAdapter {
    /// Creates an empty memory cache.
    pub fn new() -> Self {
        Self::default()
    }
}

impl CacheAdapter for MemoryCacheAdapter {
    fn get(&self, key: &CacheKey) -> CacheResult<Option<Vec<u8>>> {
        let now = SystemTime::now();
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| CacheError::adapter("memory cache lock poisoned"))?;

        if let Some(entry) = entries.get(key) {
            if entry.is_expired_at(now) {
                entries.remove(key);
                return Ok(None);
            }
            return Ok(Some(entry.value.clone()));
        }

        Ok(None)
    }

    fn set(&self, entry: CacheEntry) -> CacheResult<()> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| CacheError::adapter("memory cache lock poisoned"))?;
        entries.insert(entry.key.clone(), entry);
        Ok(())
    }

    fn delete(&self, key: &CacheKey) -> CacheResult<()> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| CacheError::adapter("memory cache lock poisoned"))?;
        entries.remove(key);
        Ok(())
    }

    fn invalidate_tag(&self, tag: &str) -> CacheResult<u64> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| CacheError::adapter("memory cache lock poisoned"))?;

        let keys: Vec<CacheKey> = entries
            .iter()
            .filter(|(_, entry)| entry.tags.iter().any(|value| value == tag))
            .map(|(key, _)| key.clone())
            .collect();

        let removed = keys.len() as u64;
        for key in keys {
            entries.remove(&key);
        }
        Ok(removed)
    }

    fn clear(&self) -> CacheResult<()> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| CacheError::adapter("memory cache lock poisoned"))?;
        entries.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::cache::Cache;

    #[test]
    fn memory_round_trip() {
        let cache = Cache::memory();
        cache
            .set_bytes(
                CacheKey::new("demo:key"),
                b"payload".to_vec(),
                &["demo"],
                None,
            )
            .unwrap();
        assert_eq!(
            cache.get_bytes(&CacheKey::new("demo:key")).unwrap(),
            Some(b"payload".to_vec())
        );
    }

    #[test]
    fn memory_expires_entries() {
        let cache = Cache::memory();
        cache
            .set_bytes(
                CacheKey::new("demo:ttl"),
                b"gone".to_vec(),
                &[],
                Some(Duration::from_millis(1)),
            )
            .unwrap();
        std::thread::sleep(Duration::from_millis(5));
        assert_eq!(cache.get_bytes(&CacheKey::new("demo:ttl")).unwrap(), None);
    }

    #[test]
    fn memory_invalidates_tag() {
        let cache = Cache::memory();
        cache
            .set_bytes(CacheKey::new("a"), vec![1], &["movies"], None)
            .unwrap();
        cache
            .set_bytes(CacheKey::new("b"), vec![2], &["movies"], None)
            .unwrap();
        cache
            .set_bytes(CacheKey::new("c"), vec![3], &["other"], None)
            .unwrap();

        assert_eq!(cache.invalidate_tag("movies").unwrap(), 2);
        assert!(cache.get_bytes(&CacheKey::new("a")).unwrap().is_none());
        assert!(cache.get_bytes(&CacheKey::new("b")).unwrap().is_none());
        assert_eq!(cache.get_bytes(&CacheKey::new("c")).unwrap(), Some(vec![3]));
    }
}
