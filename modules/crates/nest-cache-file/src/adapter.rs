//! Disk-backed cache adapter.

use std::sync::Mutex;
use std::time::SystemTime;

use nest_cache::{CacheAdapter, CacheEntry, CacheError, CacheKey, CacheResult};
use nest_file::{FileService, FileServiceConfig};
use sha2::{Digest, Sha256};

use crate::config::FileCacheConfig;
use crate::index::TagIndex;
use crate::meta::FileCacheMeta;

const DATA_DIR: &str = "data";
const META_DIR: &str = "meta";

/// Persists cache entries under `{root}/data` and `{root}/meta`.
pub struct FileCacheAdapter {
    files: FileService,
    index: Mutex<TagIndex>,
}

impl FileCacheAdapter {
    /// Creates a file cache under the configured root.
    pub fn new(config: FileCacheConfig) -> CacheResult<Self> {
        let files = FileService::with_config(
            FileServiceConfig::scoped(&config.root).allow_create_dirs(true),
        )
        .map_err(|error| CacheError::io(error.to_string()))?;

        for directory in [DATA_DIR, META_DIR] {
            if !files.exists(directory).unwrap_or(false) {
                files
                    .create_dir_all(directory)
                    .map_err(|error| CacheError::io(error.to_string()))?;
            }
        }

        let mut index = TagIndex::default();
        rebuild_index(&files, &mut index)?;

        Ok(Self {
            files,
            index: Mutex::new(index),
        })
    }

    /// Returns metadata for a cached entry when present.
    pub fn metadata_for(&self, key: &CacheKey) -> CacheResult<Option<FileCacheMeta>> {
        let hash = key_hash(key);
        read_meta(&self.files, &hash)
    }

    fn data_path(hash: &str) -> String {
        format!("{DATA_DIR}/{hash}.bin")
    }

    fn meta_path(hash: &str) -> String {
        format!("{META_DIR}/{hash}.json")
    }

    fn remove_entry(&self, hash: &str) -> CacheResult<()> {
        let _ = self.files.delete_file(Self::data_path(hash));
        let _ = self.files.delete_file(Self::meta_path(hash));
        let mut index = self
            .index
            .lock()
            .map_err(|_| CacheError::adapter("file cache index lock poisoned"))?;
        index.unregister(hash);
        Ok(())
    }
}

impl CacheAdapter for FileCacheAdapter {
    fn get(&self, key: &CacheKey) -> CacheResult<Option<Vec<u8>>> {
        let hash = key_hash(key);
        let Some(meta) = read_meta(&self.files, &hash)? else {
            return Ok(None);
        };

        if meta.is_expired_at(SystemTime::now()) {
            self.remove_entry(&hash)?;
            return Ok(None);
        }

        match self.files.read_bytes(Self::data_path(&hash)) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if is_not_found(&error) => {
                self.remove_entry(&hash)?;
                Ok(None)
            }
            Err(error) => Err(CacheError::io(error.to_string())),
        }
    }

    fn set(&self, entry: CacheEntry) -> CacheResult<()> {
        let hash = key_hash(&entry.key);
        let meta = FileCacheMeta::from_entry(&entry, None);

        self.files
            .write_bytes(Self::data_path(&hash), &entry.value)
            .map_err(|error| CacheError::io(error.to_string()))?;
        write_meta(&self.files, &hash, &meta)?;

        let mut index = self
            .index
            .lock()
            .map_err(|_| CacheError::adapter("file cache index lock poisoned"))?;
        index.unregister(&hash);
        index.register(&hash, &meta.tags);
        Ok(())
    }

    fn delete(&self, key: &CacheKey) -> CacheResult<()> {
        self.remove_entry(&key_hash(key))
    }

    fn invalidate_tag(&self, tag: &str) -> CacheResult<u64> {
        let hashes = {
            let index = self
                .index
                .lock()
                .map_err(|_| CacheError::adapter("file cache index lock poisoned"))?;
            index.hashes_for_tag(tag)
        };

        let mut removed = 0u64;
        for hash in hashes {
            self.remove_entry(&hash)?;
            removed += 1;
        }
        Ok(removed)
    }

    fn clear(&self) -> CacheResult<()> {
        for hash in list_hashes(&self.files)? {
            self.remove_entry(&hash)?;
        }

        let mut index = self
            .index
            .lock()
            .map_err(|_| CacheError::adapter("file cache index lock poisoned"))?;
        index.clear();
        Ok(())
    }
}

/// Stores bytes with an explicit content type in metadata.
pub fn set_with_content_type(
    adapter: &FileCacheAdapter,
    entry: CacheEntry,
    content_type: impl Into<String>,
) -> CacheResult<()> {
    let hash = key_hash(&entry.key);
    let meta = FileCacheMeta::from_entry(&entry, Some(content_type.into()));

    adapter
        .files
        .write_bytes(FileCacheAdapter::data_path(&hash), &entry.value)
        .map_err(|error| CacheError::io(error.to_string()))?;
    write_meta(&adapter.files, &hash, &meta)?;

    let mut index = adapter
        .index
        .lock()
        .map_err(|_| CacheError::adapter("file cache index lock poisoned"))?;
    index.unregister(&hash);
    index.register(&hash, &meta.tags);
    Ok(())
}

fn key_hash(key: &CacheKey) -> String {
    let digest = Sha256::digest(key.as_str().as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn read_meta(files: &FileService, hash: &str) -> CacheResult<Option<FileCacheMeta>> {
    match files.read_text(FileCacheAdapter::meta_path(hash)) {
        Ok(text) => {
            let meta = serde_json::from_str(&text)
                .map_err(|error| CacheError::serialization(error.to_string()).with_source(error))?;
            Ok(Some(meta))
        }
        Err(error) if is_not_found(&error) => Ok(None),
        Err(error) => Err(CacheError::io(error.to_string())),
    }
}

fn write_meta(files: &FileService, hash: &str, meta: &FileCacheMeta) -> CacheResult<()> {
    let json = serde_json::to_vec_pretty(meta)
        .map_err(|error| CacheError::serialization(error.to_string()).with_source(error))?;
    files
        .write_bytes(FileCacheAdapter::meta_path(hash), &json)
        .map_err(|error| CacheError::io(error.to_string()))
}

fn rebuild_index(files: &FileService, index: &mut TagIndex) -> CacheResult<()> {
    for hash in list_hashes(files)? {
        if let Some(meta) = read_meta(files, &hash)? {
            index.register(&hash, &meta.tags);
        }
    }
    Ok(())
}

fn list_hashes(files: &FileService) -> CacheResult<Vec<String>> {
    let entries = files
        .list_dir(META_DIR)
        .map_err(|error| CacheError::io(error.to_string()))?;

    Ok(entries
        .into_iter()
        .filter_map(|entry| entry.name.strip_suffix(".json").map(str::to_string))
        .collect())
}

fn is_not_found(error: &nest_error::NestError) -> bool {
    error
        .code()
        .is_some_and(|code| code == nest_file::codes::NEST_FILE_NOT_FOUND)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use nest_cache::Cache;
    use tempfile::tempdir;

    use super::*;
    use crate::config::FileCacheConfig;

    #[test]
    fn file_round_trip_survives_restart() {
        let dir = tempdir().unwrap();
        let key = CacheKey::new("loon:artwork:alien-1979:poster");

        {
            let cache = Cache::new(std::sync::Arc::new(
                FileCacheAdapter::new(FileCacheConfig::new(dir.path())).unwrap(),
            ));
            cache
                .set_bytes(
                    key.clone(),
                    b"poster-bytes".to_vec(),
                    &["movie:alien-1979"],
                    None,
                )
                .unwrap();
        }

        let cache = Cache::new(std::sync::Arc::new(
            FileCacheAdapter::new(FileCacheConfig::new(dir.path())).unwrap(),
        ));
        assert_eq!(
            cache.get_bytes(&key).unwrap(),
            Some(b"poster-bytes".to_vec())
        );
    }

    #[test]
    fn file_invalidates_tag() {
        let dir = tempdir().unwrap();
        let cache = Cache::new(std::sync::Arc::new(
            FileCacheAdapter::new(FileCacheConfig::new(dir.path())).unwrap(),
        ));

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
        assert_eq!(cache.get_bytes(&CacheKey::new("c")).unwrap(), Some(vec![3]));
    }

    #[test]
    fn file_expires_entries() {
        let dir = tempdir().unwrap();
        let adapter = FileCacheAdapter::new(FileCacheConfig::new(dir.path())).unwrap();
        let key = CacheKey::new("ttl");

        adapter
            .set(CacheEntry {
                key: key.clone(),
                value: b"gone".to_vec(),
                tags: vec![],
                expires_at: Some(SystemTime::now() - Duration::from_secs(60)),
            })
            .unwrap();

        assert_eq!(adapter.get(&key).unwrap(), None);
    }

    #[test]
    fn set_with_content_type_persists_metadata() {
        let dir = tempdir().unwrap();
        let adapter = FileCacheAdapter::new(FileCacheConfig::new(dir.path())).unwrap();
        let key = CacheKey::new("loon:artwork:demo:poster");

        set_with_content_type(
            &adapter,
            CacheEntry {
                key: key.clone(),
                value: b"jpeg".to_vec(),
                tags: vec!["movie:demo".into()],
                expires_at: None,
            },
            "image/jpeg",
        )
        .unwrap();

        let meta = adapter.metadata_for(&key).unwrap().expect("metadata");
        assert_eq!(meta.content_type.as_deref(), Some("image/jpeg"));
    }
}
