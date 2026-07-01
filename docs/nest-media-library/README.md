# nest-media-library

Media library scanning and indexing for the [Nest framework](../../README.md).

**Crate path:** [`modules/crates/nest-media-library`](../../modules/crates/nest-media-library)

## Role

`nest-media-library` discovers video files via [`nest-file`](../nest-file/README.md), applies filename heuristics, and optionally enriches candidates through injected [`nest-media`](../nest-media/README.md) provider traits.

It does **not** serve HTTP, call TMDB, invoke FFmpeg, or persist to SQLite directly.

| Layer | Responsibility |
|-------|----------------|
| `nest-media` | Domain models + provider trait contracts |
| **`nest-media-library`** | Library config, filesystem scan, indexing orchestration |
| `nest-tmdb` (future) | `MetadataProvider` implementation |
| `nest-transcode` | **module** | `MediaInspector` via FFprobe — [README](../nest-transcode/README.md) |
| `nest-data-sqlite` | `MediaLibraryRepository` persistence (Loon) |

## Quick start

```rust
use nest_core::AppBuilder;
use nest_file::FileModule;
use nest_media_library::{MediaLibraryConfig, MediaLibraryModule, LibraryScanner};

let mut built = AppBuilder::new()
    .module(FileModule::scoped("./media"))
    .module(MediaLibraryModule::new())
    .build()?;
built.startup()?;

let scanner = built.context.service::<LibraryScanner>()?;
let config = MediaLibraryConfig::new("main", ["Movies"]);
let result = scanner.discover(&config)?;
```

## Full indexing pipeline

```rust
use nest_media_library::{LibraryIndexer, LibraryScanOptions};

let indexer = built.context.service::<LibraryIndexer>()?;
let result = indexer
    .scan_library(&config, LibraryScanOptions::full())
    .await?;
```

Inject providers when wiring the module:

```rust
MediaLibraryModule::new()
    .with_metadata(tmdb_client)
    .with_inspector(ffprobe_inspector)
    .with_repository(sqlite_repository)
```

## Background scans

```rust
use nest_media_library::LibraryScanTask;
use nest_task_runtime::TaskManagerService;

let task = LibraryScanTask::discover(config);
task_manager.spawn(task).await?;
```

## v0.1 scope

| Included | Deferred |
|----------|----------|
| `LibraryScanner` discovery | TV show folder layouts |
| Filename title/year heuristics | NFO sidecar parsing |
| `LibraryIndexer` orchestration | Watch folders / inotify |
| `MediaLibraryModule` | `nest-config` auto-load |
| `LibraryScanTask` | Duplicate detection |

## Dependency rule

```text
nest-error  ←  nest-media
                    ↑
nest-file  ←  nest-media-library  →  nest-core, nest-task
```

## Related

- [Implementation plan](../plan/nest-media-library-v1.md)
- [nest-media](../nest-media/README.md) — domain models and provider traits
- [nest-file](../nest-file/README.md) — scoped filesystem I/O
- [Loon app](../../apps/loon/README.md)
