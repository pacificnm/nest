//! HTTP fetch with disk cache backing.

#![allow(clippy::result_large_err)]

use std::time::Duration;

use nest_cache::{Cache, CacheKey};
use nest_error::{NestError, NestResult};
use reqwest::blocking::Client;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Fetches remote images and stores bytes in a [`Cache`] adapter.
#[derive(Clone)]
pub struct ImageService {
    cache: Cache,
    http: Client,
}

impl ImageService {
    /// Creates a service backed by the given cache (memory or file adapter).
    pub fn new(cache: Cache) -> NestResult<Self> {
        let http = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .map_err(|error| NestError::network(format!("image HTTP client: {error}")))?;

        Ok(Self { cache, http })
    }

    /// Returns cached bytes or fetches from `url`, stores, and returns.
    pub fn fetch_bytes(
        &self,
        url: &str,
        key: &CacheKey,
        tags: &[&str],
    ) -> NestResult<Vec<u8>> {
        if let Some(bytes) = self.cache.get_bytes(key)? {
            tracing::debug!(url, "image cache hit");
            return Ok(bytes);
        }

        tracing::debug!(url, "image cache miss — fetching");
        let response = self.http.get(url).send().map_err(|error| {
            NestError::network(format!("image fetch failed for {url}: {error}"))
        })?;

        if !response.status().is_success() {
            return Err(NestError::network(format!(
                "image fetch failed for {url}: HTTP {}",
                response.status()
            )));
        }

        let bytes = response.bytes().map_err(|error| {
            NestError::network(format!("image read failed for {url}: {error}"))
        })?;

        if bytes.is_empty() {
            return Err(NestError::validation(format!("empty image response: {url}")));
        }

        let bytes = bytes.to_vec();
        self.cache.set_bytes(key.clone(), bytes.clone(), tags, None)?;
        Ok(bytes)
    }

    /// Removes all disk/memory cache entries with the given tag.
    pub fn invalidate_tag(&self, tag: &str) -> NestResult<u64> {
        self.cache
            .invalidate_tag(tag)
            .map_err(NestError::from)
    }

    /// Clears cached poster, backdrop, and cast images for a movie slug.
    pub fn invalidate_movie(&self, slug: &str) -> NestResult<u64> {
        self.invalidate_tag(&crate::key::movie_cache_tag(slug))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_cache::Cache;

    #[test]
    fn returns_cached_bytes_without_http() {
        let cache = Cache::memory();
        let key = CacheKey::new("cached-poster");
        cache
            .set_bytes(key.clone(), b"jpeg-bytes".to_vec(), &["artwork"], None)
            .unwrap();

        let service = ImageService::new(cache).unwrap();
        let bytes = service
            .fetch_bytes("http://127.0.0.1:1/unused", &key, &["artwork"])
            .unwrap();
        assert_eq!(bytes, b"jpeg-bytes");
    }

    #[test]
    fn invalidate_movie_removes_tagged_entries() {
        let cache = Cache::memory();
        cache
            .set_bytes(
                CacheKey::new("poster"),
                b"old-poster".to_vec(),
                &["movie:alien", "artwork"],
                None,
            )
            .unwrap();
        cache
            .set_bytes(
                CacheKey::new("other"),
                b"keep".to_vec(),
                &["artwork"],
                None,
            )
            .unwrap();

        let service = ImageService::new(cache).unwrap();
        assert_eq!(service.invalidate_movie("alien").unwrap(), 1);
        assert!(service
            .fetch_bytes("http://127.0.0.1:1/new", &CacheKey::new("poster"), &["movie:alien"])
            .is_err());
    }
}
