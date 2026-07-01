# nest-media

Media domain models and provider contracts for the [Nest framework](../../README.md).

**Crate path:** [`core/crates/nest-media`](../../core/crates/nest-media)

## Role

`nest-media` defines **what media is** in Nest — movies, tracks, artwork, metadata search types, and provider traits. It does not perform HTTP, database I/O, filesystem scanning, FFmpeg inspection, or TMDB calls.

| Layer | Responsibility |
|-------|----------------|
| `nest-media` | Domain models + provider trait contracts |
| `nest-media-library` | Scanning and library indexing — [README](../nest-media-library/README.md) |
| `nest-tmdb` | `MetadataProvider` implementation — [README](../nest-tmdb/README.md) |
| `nest-transcode` | `MediaInspector` via FFprobe — [README](../nest-transcode/README.md) |
| `nest-data-sqlite` | `MediaLibraryRepository` persistence (Loon schema) |
| `nest-stream` (future) | Byte-range video streaming |

## Quick start

```rust
use nest_media::{MediaId, Movie, MediaItem, MediaKind};

let movie = Movie::new(MediaId::new("movie-1"), "Alien");
let item = MediaItem::from_movie(&movie);
assert_eq!(item.kind, MediaKind::Movie);
```

## Core models (v0.1 — movies)

- `MediaId`, `ExternalMediaId`, `MediaKind`
- `Movie`, `MediaItem`, `PersonCredit`
- `MediaTracks`, `VideoTrack`, `AudioTrack`, `SubtitleTrack`
- `Artwork`, `ArtworkKind`, `ArtworkSource`
- `ExternalIds`
- `MovieSearchQuery`, `MovieSearchResult`, `MovieMetadata`
- `MediaInput`, `MediaInspection`

## Provider traits

| Trait | Implemented by |
|-------|----------------|
| `MetadataProvider` | `nest-tmdb` (future) |
| `MediaInspector` | `nest-transcode` — [plan](../plan/nest-transcode-v1.md) |
| `MediaLibraryRepository` | `nest-data-sqlite` (Loon) |

Traits are async and behind the default `async` feature.

## Dependency rule

```text
nest-error  ←  nest-media
```

`nest-media` does **not** depend on `nest-core`, HTTP crates, `nest-file`, or database crates.

## Related

- [Implementation plan](../plan/nest-media-v1.md)
- [nest-media-library README](../nest-media-library/README.md) — scanning and indexing
- [nest-tmdb README](../nest-tmdb/README.md) — TMDB metadata provider
- [nest-transcode README](../nest-transcode/README.md) — FFprobe media inspection
- [Loon app](../../apps/loon/README.md)
- [nest-data](../nest-data/README.md) — similar contracts-only pattern
