# nest-image

Remote image **fetch and disk cache** for Nest applications.

**Crate path:** [`core/crates/nest-image`](../../core/crates/nest-image)

## Desktop (Tauri + React)

| Layer | Status |
|-------|--------|
| `ImageService`, `ImageModule`, cache keys | **Keep** — register in `src-tauri/` |
| egui `RemoteImage` widget (`egui` feature) | **Legacy** — do not use in new apps |
| React display | **Planned** — `<RemoteImage>` in `ui/` via Tauri IPC |

See [nest-react-ui v1 plan](../plan/nest-react-ui-v1.md).

## Quick start (Rust)

```rust
use nest_cache::Cache;
use nest_image::{ImageModule, ImageService};

let cache = Cache::file("/var/cache/my-app")?;
let built = AppBuilder::new()
    .module(ImageModule::with_cache(cache))
    .build()?;

let images = built.context.service::<ImageService>()?;
let bytes = images.fetch_bytes(
    "https://example.com/poster.jpg",
    &cache_key_for_url("https://example.com/poster.jpg"),
    &["artwork"],
)?;
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| *(none)* | yes | `ImageService`, module, cache helpers only |
| `egui` | no | Legacy egui texture widget — deprecated |

```toml
nest-image = { workspace = true }                          # desktop / headless
nest-image = { workspace = true, features = ["egui"] }   # legacy Kiwi only
```

## Related

- [nest-react-ui v1 plan](../plan/nest-react-ui-v1.md)
- [nest-cache](../nest-cache/README.md)
- [nest-tauri](../nest-tauri/README.md)
