# Nest Shell

**Nest Shell** is a desktop **demo** for the [Nest framework](../README.md). It is a Tauri + React app that shows a KDE-inspired environment inside one native window: wallpaper, icons, start menu, taskbar, window manager, and built-in shell apps.

In **development**, apps under `apps/` with `mode = "embed"` in `nest-app.toml` load their Vite dev server inside shell windows (UI only — no product Rust IPC). Product binaries (Kiwi, Swift standalone) remain separate.

## What it is

| Piece | Role |
|-------|------|
| **Nest Shell** | Demo desktop UI + window manager + Help Center |
| **Help** | Built-in docs browser (`docs/` + root README) |
| **Embed apps** | Dev-mode UI from `apps/*/ui` via per-app Vite (iframe) |
| **Other registry entries** | Placeholder windows |

```text
npm run tauri:dev
  ├─ shell Vite     :5173
  ├─ swift Vite     :5174   (if mode = embed)
  ├─ kiwi Vite      :5175   (if mode = embed)
  └─ open in shell window → http://localhost:<port>
```

## Built-in shell apps

| App | Status |
|-----|--------|
| **Help** | Docs TOC + markdown viewer |
| Settings | Planned |
| Terminal | Planned |

## Dev embed setup

1. Add to `apps/<name>/nest-app.toml`:

```toml
[shell.launch]
mode = "embed"

[shell.dev]
port = 5174   # unique per app

[shell.build]
entry = "ui/dist/index.html"
```

2. Run from `ui/`:

```bash
npm run tauri:dev
```

`dev:all` starts the shell plus one Vite server per embed app. Ports are written to `ui/.embed-dev-ports.json`.

**Note:** Embed mode shows the React UI only. Tauri commands from the product `src-tauri/` do not run — use each app's own `tauri dev` for full backend testing.

## App registry

The shell scans `apps/*/nest-app.toml` at startup. Apps without `mode = "embed"` open as placeholders.

Template: [`templates/desktop/nest-app.toml`](../templates/desktop/nest-app.toml)

## Current status

| Area | Status |
|------|--------|
| Desktop + icons | Done |
| Start menu + taskbar | Done |
| Window manager | Done |
| Help app | Done |
| Dev embed (Vite) | Done |
| App registry | Done |

## Quick start

```bash
cd ui
npm install
npm run tauri:dev
```

| Command | Description |
|---------|-------------|
| `npm run dev` | Shell Vite only |
| `npm run dev:all` | Shell + embed app Vite servers |
| `npm run tauri:build` | Release bundle |

## Related

- [Nest architecture](../docs/architecture.md)
- [nest-tauri](../docs/nest-tauri/README.md)
- [Desktop template](../templates/desktop/)
