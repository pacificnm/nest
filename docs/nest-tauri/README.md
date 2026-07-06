# nest-tauri

Desktop host for the [Nest framework](../../README.md).

**Frontend platform:** **Tauri + React + TypeScript + Tailwind** — see [Desktop frontend platform](../architecture.md#desktop-frontend-platform).

**Crate path:** [`core/crates/nest-tauri`](../../core/crates/nest-tauri)

**Status:** Scaffolded (v1). Enable the `runtime` feature in app `Cargo.toml` to link Tauri and call `.run(tauri::generate_context!())`.

`nest-tauri` owns Tauri startup, the command/event bridge between Rust and the webview, and window lifecycle. Product UI lives in `ui/` (React + Tailwind); this crate does not own CSV, HTTP, or domain logic beyond host initialization.

## Why this stack

- **Tauri:** Rust modules in-process; Nest `AppContext` available to Tauri commands
- **React:** Component model, ecosystem, tooling for complex desktop UX
- **Tailwind:** Utility styling wired to `nest-design` tokens via `nest-react-theme`

## Quick start

```rust
// src-tauri/src/main.rs
use nest_tauri::TauriApp;
use nest_theme::ThemeModule;

fn main() {
    TauriApp::new("my-app")
        .module(ThemeModule::default())
        .run(tauri::generate_context!());
}
```

App `Cargo.toml`:

```toml
nest-tauri = { workspace = true, features = ["runtime", "images"] }
```

Enable `images` when registering [`ImageModule`](../nest-image/README.md) and using [`RemoteImage`](../../templates/desktop/ui/src/components/RemoteImage.tsx) in React.

```tsx
// ui/src/App.tsx
import { invoke } from "@tauri-apps/api/core";

export function App() {
  return <h1 className="text-nest-foreground">Hello, Nest</h1>;
}
```

## App layout

| Path | Stack |
|------|--------|
| `ui/` | React + TypeScript + Tailwind (Vite) |
| `src-tauri/` | Tauri + `nest-tauri` + Nest modules |

## Config

```toml
[tauri]
title = "My App"
width = 1280
height = 800
```

CLI flags override config; config overrides defaults.

## Logging

Desktop hosts default to **file-only** logging (no stdout while the window is active), consistent with `nest-tui`.

## Theme

Register `ThemeModule` in Rust. Use [`nest-react-theme`](../nest-react-theme/README.md) in `ui/` to apply tokens as CSS variables and Tailwind utilities. Built-in command `nest_theme_css` returns the active theme for the webview.

## Built-in IPC commands

| Command | Returns | Feature |
|---------|---------|---------|
| `nest_app_metadata` | App name and window title | `runtime` |
| `nest_theme_css` | Active theme CSS variables and `:root` block | `runtime` |
| `nest_image_fetch` | Base64 image bytes, MIME type, cache key | `runtime` + `images` |
| `nest_image_invalidate_tag` | Count of removed cache entries | `runtime` + `images` |

Register `ImageModule` in `src-tauri/` before calling image commands. See [desktop template](../../templates/desktop/).

## Related

- [Desktop template](../../templates/desktop/)
- [nest-react-ui v1 plan](../plan/nest-react-ui-v1.md)
- [nest-react-theme](../nest-react-theme/README.md)
- [nest-tui](../nest-tui/README.md) — terminal host
