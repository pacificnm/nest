# nest-tmdb

TMDB metadata provider adapter for the [Nest framework](../../README.md).

**Crate path:** [`modules/crates/nest-tmdb`](../../modules/crates/nest-tmdb)

## Role

`nest-tmdb` translates TMDB API responses into [`nest-media`](../nest-media/README.md) types and implements [`MetadataProvider`](../nest-media/README.md). It does not own media domain models, serve HTTP, scan libraries, or persist to SQLite.

| Layer | Responsibility |
|-------|----------------|
| `nest-media` | Domain models + `MetadataProvider` trait |
| **`nest-tmdb`** | TMDB HTTP client + metadata mapping |
| `nest-media-library` | Injects metadata provider during library indexing |

## Quick start

```rust
use nest_core::AppBuilder;
use nest_http_client::HttpClientModule;
use nest_media::MovieSearchQuery;
use nest_tmdb::{TmdbConfig, TmdbMetadataProvider, TmdbModule};

let built = AppBuilder::new()
    .module(HttpClientModule::default())
    .module(TmdbModule::with_config(
        TmdbConfig::builder().api_key("your-key").build()?,
    ))
    .build()?;
built.startup()?;

let provider = built.context.service::<TmdbMetadataProvider>()?;
let results = provider
    .search_movie(MovieSearchQuery::new("Alien").with_year(1979))
    .await?;
let metadata = provider.get_movie(results[0].external_id.clone()).await?;
```

## Configuration

Environment variable (default `TMDB_API_KEY`):

```bash
export TMDB_API_KEY="..."
```

Or TOML via `nest-config`:

```toml
[tmdb]
api_key_env = "TMDB_API_KEY"
language = "en-US"
region = "US"
```

## Image URLs

Use [`TmdbImageService`](../../modules/crates/nest-tmdb/src/images.rs) to build poster and backdrop URLs from TMDB file paths. `MovieMetadata` does not include artwork in nest-media v0.1 — merge artwork when converting to `Movie`.

```rust
use nest_tmdb::{ImageSize, TmdbImageService};

let images = built.context.service::<TmdbImageService>()?;
let poster = images.poster_url("/abc.jpg", ImageSize::W500).await;
```

## External ids

TMDB movie ids use the nest-media convention `tmdb:{id}` (e.g. `tmdb:348`).

## v0.1 scope

| Included | Deferred |
|----------|----------|
| Movie search | TV shows |
| Movie details + credits + external ids | `append_to_response` batching |
| Poster/backdrop URL builders | Shared `nest-cache` module |
| `TmdbModule` | Trending, persons, reviews |

## Dependency rule

```text
nest-error  ←  nest-media
                    ↑
nest-http-client  ←  nest-tmdb  →  nest-core
```

## Related

- [Implementation plan](../plan/nest-tmdb-v1.md)
- [nest-media](../nest-media/README.md) — domain models and provider traits
- [nest-media-library](../nest-media-library/README.md) — library indexing
- [Loon app](../../apps/loon/README.md)
