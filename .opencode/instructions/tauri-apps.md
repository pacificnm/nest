# Desktop apps (Tauri)

Every Nest **desktop** product follows one runtime model: business logic in
Rust, `nest-tauri` as the host, React as presentation only, IPC only at the
webview boundary.

| Layer | Technology | Location |
|-------|------------|----------|
| Shell | **Tauri** | `src-tauri/`, `nest-tauri` |
| UI | **React** + **TypeScript** | `ui/src/` (Vite) |
| Styling | **Tailwind CSS** | `ui/tailwind.config.ts`, `nest-react-theme` |
| Icons | React icon library (Lucide, Font Awesome) | `ui/` — not `nest-icon` (legacy egui) |
| Remote images | `nest-image` + React | Rust `ImageService` + `<RemoteImage>` in `ui/` |
| Tokens | `nest-design` / `nest-theme` | Rust services → CSS variables in the webview |

## Folder layout

```text
my-app/
├── ui/                   # React + TypeScript + Tailwind (Vite)
│   ├── src/
│   ├── tailwind.config.ts
│   └── package.json
├── src-tauri/
│   ├── src/
│   │   ├── main.rs       # TauriApp::new("my-app").module(…).run()
│   │   └── commands/     # thin #[tauri::command] wrappers
│   ├── Cargo.toml
│   └── tauri.conf.json
└── crates/core/          # shared product logic (recommended)
```

## Command surface pattern

Implement each product operation **once** in `crates/core`, then wire a thin
adapter per host:

```rust
// crates/core — domain logic
pub async fn scan_library(ctx: &AppContext) -> NestResult<ScanReport> {
    let svc = ctx.get::<LibraryService>()?;
    svc.scan().await
}

// src-tauri — host adapter (IPC)
#[tauri::command]
async fn scan_library_cmd(state: State<'_, AppState>) -> Result<ScanReport, String> {
    scan_library(state.context()).await.map_err(|e| e.to_string())
}
```

Built-in Tauri commands from `nest-tauri` (when the `runtime` feature is
enabled): `nest_app_metadata`, `nest_theme_css`, `nest_image_fetch`,
`nest_image_invalidate_tag`.

**Legacy — do not use for new work:** `nest-gui` (egui), `nest-icon` (egui),
the `nest-image` egui widget. Use `nest-tauri` + React `ui/` instead.

For MCP knowledge-search requirements and IPC do/don't rules while writing
this code, see [desktop-ui.md](desktop-ui.md).

See [docs/app-standard.md](../../docs/app-standard.md) and
[docs/nest-tauri/README.md](../../docs/nest-tauri/README.md).
