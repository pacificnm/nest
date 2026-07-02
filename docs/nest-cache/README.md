# nest-cache

Cache contracts for the [Nest framework](../../README.md).

**Crate path (planned):** [`core/crates/nest-cache`](../../core/crates/nest-cache)

## Status

**Implemented** — see [implementation plan](../plan/nest-cache-v1.md).

## Role

`nest-cache` defines **what caching means** in Nest: keyed entries, optional TTL, tags for grouped invalidation, and a pluggable storage adapter. It does not choose Redis vs disk vs RAM — adapter crates do.

| Layer | Responsibility |
|-------|----------------|
| **`nest-cache`** | **core** — `Cache`, `CacheKey`, tags, invalidate API, `CacheModule` |
| `nest-cache-memory` (planned) | In-process adapter — shipped as `MemoryCacheAdapter` in core for v1 |
| [`nest-cache-file`](../plan/nest-cache-file-v1.md) | **module** — disk-backed entries (e.g. Loon artwork, TMDB JSON blobs) |
| `nest-cache-redis` (planned) | **module** — shared/distributed cache |

Follows the same split as [`nest-data`](../nest-data/README.md) (contracts in core, I/O in adapters).

## Design principles

1. **One cache API** — apps call `Cache::get` / `set` / `delete` / `invalidate_tag`; not separate caches per backend.
2. **Keys** — opaque `CacheKey` (string namespace + id); callers define namespaces (`tmdb:config`, `loon:poster:{id}`).
3. **Tags** — zero or more labels per entry; `invalidate_tag("movies")` drops all matching keys.
4. **Adapters** — `CacheAdapter` trait; memory, file, and Redis are interchangeable at wiring time.
5. **No app knowledge in core** — `nest-cache` must not know about Loon, TMDB, or HTTP.

## API sketch (v1 target)

```rust
pub struct CacheKey(String);

pub struct CacheEntry {
    pub key: CacheKey,
    pub value: Vec<u8>,
    pub tags: Vec<String>,
    pub expires_at: Option<SystemTime>,
}

pub trait CacheAdapter: Send + Sync {
    fn get(&self, key: &CacheKey) -> CacheResult<Option<Vec<u8>>>;
    fn set(&self, entry: CacheEntry) -> CacheResult<()>;
    fn delete(&self, key: &CacheKey) -> CacheResult<()>;
    fn invalidate_tag(&self, tag: &str) -> CacheResult<u64>;
    fn clear(&self) -> CacheResult<()>;
}

pub struct Cache {
    adapter: Arc<dyn CacheAdapter>,
}

impl Cache {
    pub fn get<T: DeserializeOwned>(&self, key: &CacheKey) -> CacheResult<Option<T>> { /* ... */ }
    pub fn set<T: Serialize>(&self, key: CacheKey, value: &T, tags: &[&str], ttl: Option<Duration>) -> CacheResult<()> { /* ... */ }
    pub fn invalidate_tag(&self, tag: &str) -> CacheResult<u64> { /* ... */ }
}
```

## Example usage (future)

```rust
use nest_core::AppBuilder;
use nest_cache::{Cache, CacheKey, CacheModule};
use nest_cache_memory::MemoryCacheAdapter;

let cache = Cache::new(Arc::new(MemoryCacheAdapter::new()));
cache.set(
    CacheKey::new("tmdb:configuration"),
    &config,
    &["tmdb"],
    Some(Duration::from_secs(86400)),
)?;

// Later: drop all TMDB-derived entries
cache.invalidate_tag("tmdb")?;
```

## Consumers (planned)

| Consumer | Use case |
|----------|----------|
| [Loon](../../apps/loon/docs/v1.md) | Local poster/backdrop file cache (`nest-cache-file`) |
| [nest-tmdb](../nest-tmdb/README.md) | `/configuration` and metadata response cache |
| Future HTTP modules | Response memoization with tag invalidation |

## Related

- [Implementation plan](../plan/nest-cache-v1.md)
- [nest-data](../nest-data/README.md) — similar contracts + adapter pattern
- [nest-file](../nest-file/README.md) — scoped I/O for file cache adapter
