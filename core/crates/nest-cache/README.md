# nest-cache

Cache contracts for the [Nest framework](../../README.md).

**Crate path:** [`core/crates/nest-cache`](.)

## Role

Keyed entries, optional TTL, tags for grouped invalidation, and a pluggable [`CacheAdapter`](src/adapter.rs) trait.

| Layer | Responsibility |
|-------|----------------|
| **`nest-cache`** | **core** — `Cache`, `CacheKey`, tags, `MemoryCacheAdapter`, `CacheModule` |
| [`nest-cache-file`](../../../modules/crates/nest-cache-file) | **module** — disk-backed adapter |

## Quick start

```rust
use std::sync::Arc;
use std::time::Duration;

use nest_cache::{Cache, CacheKey, CacheModule, MemoryCacheAdapter};

let cache = Cache::new(Arc::new(MemoryCacheAdapter::new()));
cache.set_bytes(
    CacheKey::new("tmdb:configuration"),
    br#"{"images":{"base_url":"https://image.tmdb.org/t/p/"}}"#.to_vec(),
    &["tmdb"],
    Some(Duration::from_secs(86_400)),
)?;

let removed = cache.invalidate_tag("tmdb")?;
```

## Documentation

| Document | Description |
|----------|-------------|
| [docs/nest-cache/README.md](../../../docs/nest-cache/README.md) | Overview |
| [docs/plan/nest-cache-v1.md](../../../docs/plan/nest-cache-v1.md) | Implementation plan |
