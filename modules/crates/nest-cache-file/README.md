# nest-cache-file

Disk-backed [`CacheAdapter`](../../core/crates/nest-cache/src/adapter.rs) for the [Nest framework](../../README.md).

**Crate path:** [`modules/crates/nest-cache-file`](.)

## Role

Persists cache payloads under `{root}/data/{sha256}.bin` with JSON metadata in `{root}/meta/{sha256}.json`. Uses [`nest-file`](../../core/crates/nest-file) for scoped I/O.

Primary consumer: **Loon artwork cache** → `GET /api/artwork/:slug/:kind` (next step).

## Quick start

```rust
use std::sync::Arc;

use nest_cache::{Cache, CacheKey};
use nest_cache_file::{FileCacheAdapter, FileCacheConfig, FileCacheModule};

let cache = Cache::new(Arc::new(
    FileCacheAdapter::new(FileCacheConfig::new("/var/lib/loon/cache"))?,
));

cache.set_bytes(
    CacheKey::scoped("loon", &["artwork", "alien-1979", "poster"]),
    image_bytes,
    &["movie:alien-1979", "artwork"],
    None,
)?;
```

## Related

- [nest-cache](../../core/crates/nest-cache/README.md)
- [nest-cache-file v1 plan](../../docs/plan/nest-cache-file-v1.md)
