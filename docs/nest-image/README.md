# nest-image

Remote image **fetch and disk cache** for Nest applications.

**Crate path:** [`core/crates/nest-image`](../../core/crates/nest-image)

## Desktop (Tauri + React)

| Layer | Status |
|-------|--------|
| `ImageService`, `ImageModule`, cache keys | **Keep** — register in `src-tauri/` |
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

The crate is now render-agnostic: it fetches and caches bytes only. Desktop apps
display cached bytes in the React webview via Tauri IPC.

```toml
nest-image = { workspace = true }   # desktop / headless
```

## Related

- [nest-react-ui v1 plan](../plan/nest-react-ui-v1.md)
- [nest-cache](../nest-cache/README.md)
- [nest-tauri](../nest-tauri/README.md)
