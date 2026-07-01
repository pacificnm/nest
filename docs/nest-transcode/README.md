# nest-transcode

FFprobe media inspection for the [Nest framework](../../README.md).

**Crate path:** [`modules/crates/nest-transcode`](../../modules/crates/nest-transcode)

## Role

`nest-transcode` probes local media files with FFprobe and implements [`nest-media`](../nest-media/README.md) `MediaInspector`. v0.1 is **inspection-only**; FFmpeg transcode jobs are deferred to v0.2.

| Layer | Responsibility |
|-------|----------------|
| `nest-media` | Domain models + `MediaInspector` trait |
| **`nest-transcode`** | FFprobe execution + track mapping |
| `nest-media-library` | Injects inspector during library indexing |

## Quick start

```rust
use nest_core::AppBuilder;
use nest_file::FileModule;
use nest_media::{MediaInput, MediaInspector};
use nest_transcode::{TranscodeConfig, TranscodeModule, FfprobeMediaInspector};

let built = AppBuilder::new()
    .module(FileModule::scoped("./media"))
    .module(TranscodeModule::with_config(
        TranscodeConfig::builder().build()?,
    ))
    .build()?;
built.startup()?;

let inspector = built.context.service::<FfprobeMediaInspector>()?;
let inspection = inspector
    .inspect(MediaInput::LocalPath("Movies/Alien.mkv".into()))
    .await?;
```

## Configuration

Environment variable (optional override):

```bash
export FFPROBE_PATH="/usr/bin/ffprobe"
```

Or TOML via `nest-config`:

```toml
[transcode]
ffprobe_path = "/usr/bin/ffprobe"
timeout_seconds = 120
```

## Library indexing

```rust
MediaLibraryModule::new()
    .with_inspector(ffprobe_inspector)
    .with_metadata(tmdb_provider)
```

When `LibraryScanOptions.inspect_files` is true, the indexer calls FFprobe for each discovered file.

## v0.1 scope

| Included | Deferred |
|----------|----------|
| FFprobe JSON probe | FFmpeg transcode jobs |
| Video/audio/subtitle mapping | HLS packaging |
| HDR detection (basic) | Thumbnail generation |
| `TranscodeModule` | Transcode progress events |

## Requirements

FFprobe must be installed and on `PATH` (or configured via `FFPROBE_PATH`).

## Dependency rule

```text
nest-error  ←  nest-media
                    ↑
nest-file  ←  nest-transcode  →  nest-core
```

## Related

- [Implementation plan](../plan/nest-transcode-v1.md)
- [nest-media](../nest-media/README.md) — domain models and provider traits
- [nest-media-library](../nest-media-library/README.md) — library indexing
- [Loon app](../../apps/loon/README.md)
