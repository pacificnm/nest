//! Typed cache facade.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::adapter::{CacheAdapter, MemoryCacheAdapter};
use crate::entry::CacheEntry;
use crate::error::{CacheError, CacheResult};
use crate::key::CacheKey;

/// Application-facing cache API.
#[derive(Clone)]
pub struct Cache {
    adapter: Arc<dyn CacheAdapter>,
}

impl Cache {
    /// Creates a cache over the given adapter.
    pub fn new(adapter: Arc<dyn CacheAdapter>) -> Self {
        Self { adapter }
    }

    /// Creates an in-memory cache for tests and single-process apps.
    pub fn memory() -> Self {
        Self::new(Arc::new(MemoryCacheAdapter::new()))
    }

    /// Returns raw bytes for a key.
    pub fn get_bytes(&self, key: &CacheKey) -> CacheResult<Option<Vec<u8>>> {
        self.adapter.get(key)
    }

    /// Stores raw bytes with optional tags and TTL.
    pub fn set_bytes(
        &self,
        key: CacheKey,
        value: Vec<u8>,
        tags: &[&str],
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        self.adapter.set(build_entry(key, value, tags, ttl))
    }

    /// Deserializes JSON bytes when present.
    pub fn get_json<T: DeserializeOwned>(&self, key: &CacheKey) -> CacheResult<Option<T>> {
        match self.get_bytes(key)? {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes).map_err(|error| {
                    CacheError::serialization(error.to_string()).with_source(error)
                })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Serializes and stores JSON with optional tags and TTL.
    pub fn set_json<T: Serialize>(
        &self,
        key: CacheKey,
        value: &T,
        tags: &[&str],
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        let bytes = serde_json::to_vec(value)
            .map_err(|error| CacheError::serialization(error.to_string()).with_source(error))?;
        self.set_bytes(key, bytes, tags, ttl)
    }

    /// Deletes one entry.
    pub fn delete(&self, key: &CacheKey) -> CacheResult<()> {
        self.adapter.delete(key)
    }

    /// Deletes every entry tagged with `tag`.
    pub fn invalidate_tag(&self, tag: &str) -> CacheResult<u64> {
        self.adapter.invalidate_tag(tag)
    }

    /// Deletes every entry.
    pub fn clear(&self) -> CacheResult<()> {
        self.adapter.clear()
    }
}

fn build_entry(key: CacheKey, value: Vec<u8>, tags: &[&str], ttl: Option<Duration>) -> CacheEntry {
    CacheEntry {
        key,
        value,
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
        expires_at: ttl.map(|duration| SystemTime::now() + duration),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
    struct Demo {
        value: u32,
    }

    #[test]
    fn json_round_trip() {
        let cache = Cache::memory();
        cache
            .set_json(
                CacheKey::new("demo:json"),
                &Demo { value: 42 },
                &["demo"],
                None,
            )
            .unwrap();
        let value: Demo = cache
            .get_json(&CacheKey::new("demo:json"))
            .unwrap()
            .expect("cached json");
        assert_eq!(value, Demo { value: 42 });
    }
}
