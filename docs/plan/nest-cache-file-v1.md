# nest-cache-file v1 Implementation Plan

## Status: Implemented

Disk-backed cache adapter for [nest-cache v1](nest-cache-v1.md). Primary consumer: **Loon poster/backdrop cache** ([Loon v1](../../apps/loon/docs/v1.md)).

Depends on **nest-cache core** being implemented first.

## Context

v0.1 Loon serves TMDB image URLs directly. v0.2+ may cache bytes locally for:

- Faster repeat loads on webOS
- Resilience when TMDB CDN is slow
- Optional `GET /api/artwork/:slug/:kind` proxy ([api-v0.2](../../apps/loon/docs/api-v0.2.md))

**Design principle:** `nest-cache-file` is a **`CacheAdapter` implementation** — not Loon-specific. Uses scoped paths via `nest-file`.

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-cache` | `CacheAdapter` trait, `CacheKey`, tags |
| **`nest-cache-file`** | **module** — persist entries under a cache root |
| `nest-file` | Scoped writes; prevent path escape |
| Loon | Chooses cache root `{data_dir}/cache`, tags entries `movie:{slug}` |

## Storage layout

```text
{cache_root}/
├── data/
│   └── {sha256(key)}.bin       # payload bytes
└── meta/
    └── {sha256(key)}.json      # tags, expires_at, content_type
```

Tag index (v0.1): in-memory `HashMap<tag, Vec<key_hash>>` rebuilt on startup by scanning `meta/`.

## Entry metadata

```json
{
  "key": "loon:artwork:alien-1979:poster:w500",
  "tags": ["movie:alien-1979", "artwork"],
  "expires_at": null,
  "content_type": "image/jpeg",
  "created_at": 1719878400
}
```

## Public API (v1 target)

```rust
pub struct FileCacheConfig {
    pub root: PathBuf,
    pub max_bytes: Option<u64>,   // LRU eviction when exceeded
}

pub struct FileCacheAdapter { /* ... */ }

impl CacheAdapter for FileCacheAdapter {
    fn get(&self, key: &CacheKey) -> CacheResult<Option<Vec<u8>>>;
    fn set(&self, entry: CacheEntry) -> CacheResult<()>;
    fn delete(&self, key: &CacheKey) -> CacheResult<()>;
    fn invalidate_tag(&self, tag: &str) -> CacheResult<u64>;
    fn clear(&self) -> CacheResult<()>;
}

pub struct FileCacheModule;
```

Register via `CacheModule::new(Arc::new(FileCacheAdapter::new(config)?))`.

## Loon usage

```toml
[loon]
data_dir = "/var/lib/loon"

[cache]
enabled = true
root = "cache"   # relative to data_dir
max_mb = 512
```

```rust
// On first poster request — fetch TMDB URL, store in cache
cache.set_bytes(
    CacheKey::new(format!("loon:artwork:{slug}:poster:w500")),
    image_bytes,
    &["movie:{slug}", "artwork"],
    None,
)?;

// On library rescan — invalidate movie artwork
cache.invalidate_tag(&format!("movie:{slug}"))?;
```

## v1 scope

### Ship

- `FileCacheAdapter` implementing `CacheAdapter`
- Payload + sidecar JSON metadata
- Tag invalidation (in-memory index + meta scan on startup)
- `FileCacheModule` or factory on `CacheModule`
- Unit tests: set/get/delete, invalidate_tag
- Integration: temp dir, concurrent sets

### Deferred

| Feature | Target |
|---------|--------|
| LRU eviction by `max_bytes` | v1.1 |
| Async adapter | v1.1 |
| Symlink/hardlink dedup | later |
| nest-cache-redis | separate plan |

## Workspace layout (target)

```text
modules/crates/nest-cache-file/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── adapter.rs
    ├── meta.rs
    ├── index.rs
    └── module.rs
```

## Testing

| Test | Type |
|------|------|
| Round-trip set/get | Unit |
| Expired entry ignored on get | Unit |
| invalidate_tag removes tagged only | Unit |
| Path traversal rejected | Unit |

## Follow-up

- Implement after [nest-cache v1](nest-cache-v1.md) core lands
- Wire Loon `GET /api/artwork/:slug/:kind` when enabled

## Related

- [nest-cache v1](nest-cache-v1.md)
- [Loon api-v0.2 artwork route](../../apps/loon/docs/api-v0.2.md)
- [nest-file README](../nest-file/README.md)
