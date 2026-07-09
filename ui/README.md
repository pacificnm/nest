# Nest Shell

**Nest Shell** is the reference desktop runtime for the [Nest framework](../README.md). It is a Tauri + React application that presents Nest as a KDE-inspired desktop environment: wallpaper, icons, start menu, taskbar, window manager, and launchable Nest apps.

It does **not** replace the operating system. Within a single native Tauri window, users get a lightweight desktop session that hosts Nest applications as managed windows.

## Platform story

| Piece | Role |
|-------|------|
| **Nest** | Modular application framework (config, services, logging, themes, validation, lifecycle) |
| **Nest Shell** | Desktop runtime and reference implementation of the platform |
| **Kiwi** | Agent IDE and app builder — creates new Nest apps |
| **Swift** | Example business/productivity app |
| **Help Center** | Docs reader for framework, shell, and registered apps |

Workflow we are building toward:

```text
Kiwi creates app  →  app registers with Nest  →  Nest Shell shows it in Start Menu  →  user launches it like a desktop app
```

**Kiwi becomes the builder. Nest Shell becomes the runtime.**

## Goals

- Demonstrate Nest as a modular app framework inside a real desktop UX.
- Provide a KDE Plasma–like shell: desktop, panel, start menu, notifications, workspaces.
- Launch multiple Nest apps from a shared environment.
- Prove reusable Nest services, config, themes, logging, validation, and app lifecycle.
- Show that Nest apps can run as desktop modules inside a Tauri frontend.

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│  React (Nest Shell UI)                                      │
│  Desktop · Taskbar · Start Menu · Window Manager · Apps     │
└───────────────────────────┬─────────────────────────────────┘
                            │ Tauri commands / events
┌───────────────────────────▼─────────────────────────────────┐
│  Tauri host (src-tauri/)                                    │
│  OS integration · security · process lifecycle              │
└───────────────────────────┬─────────────────────────────────┘
                            │ Nest services (planned)
┌───────────────────────────▼─────────────────────────────────┐
│  Nest framework                                             │
│  App registry · config · themes · logging · validation      │
└─────────────────────────────────────────────────────────────┘
```

React owns the desktop UI. Tauri owns the native host and secure OS integration. Nest owns the backend framework, app registry, configuration, services, logging, validation, and command layer.

## Shell components

### Desktop

Wallpaper/background, desktop icons, single/multi-select, double-click to launch, right-click context menus, drag icons, snap-to-grid, and optional virtual desktops.

### Taskbar / panel

KDE-style bottom panel:

```text
+---------------------------------------------------------------+
| Start | Search | Kiwi | Swift | Files |           tray  clock |
+---------------------------------------------------------------+
```

Pinned apps, running apps, window previews, clock, system tray, notifications, workspace switcher.

### Start menu

Search-as-you-type launcher with favorites and all applications:

```text
+------------------------------------------------+
| Search...                                      |
+------------------------------------------------+
| Favorites                                      |
|  Kiwi · Swift · Files · Settings · Terminal    |
| All Applications >                             |
+------------------------------------------------+
```

### Window manager

Each registered Nest app opens in a managed window inside the shell:

```text
+--------------------------------------------+
| Kiwi                              _  □  X  |
+--------------------------------------------+
|              app content                   |
+--------------------------------------------+
```

Planned behavior: drag, resize, minimize, maximize, restore, z-order, focus, snap left/right, cascade, tile.

Internal model (planned):

```rust
Window {
    id,
    title,
    app_id,
    x, y, width, height,
    minimized, maximized, focused,
    z_index,
}
```

The React frontend renders every window from this state.

### App registry

The shell should not hardcode Kiwi or Swift. Apps register through a manifest-driven registry:

```text
Nest Shell
  ├─ Start Menu
  ├─ Window Manager
  ├─ App Registry
  ├─ Notification Center
  ├─ Settings
  ├─ Help Center
  └─ Launchable Nest Apps
       ├─ Kiwi
       ├─ Swift
       ├─ Docs / Help
       ├─ Terminal
       ├─ File Manager
       └─ future apps built with Kiwi
```

Example manifest (planned):

```toml
# apps/my-app/nest-app.toml
[shell]
name = "My App"
category = "Development"
icon = "fa-solid fa-cube"
description = "Short description for the launcher and window."
# visible = true
```

Nest Shell scans `apps/*/nest-app.toml` at startup. **Help** is a built-in shell app (not under `apps/`). Copy the template from [`templates/desktop/nest-app.toml`](../templates/desktop/nest-app.toml) into new product repos.

### Adding a product app

1. Clone or create the app under `apps/<name>/`.
2. Add `nest-app.toml` at the app root (see [`templates/desktop/nest-app.toml`](../templates/desktop/nest-app.toml)).
3. Restart Nest Shell — the app appears on the desktop and in the Start menu.

Only directories with a valid `nest-app.toml` are shown. **Help** is always available as a built-in shell app.

Rust-side registration (target shape):

```rust
trait NestApp {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn icon(&self) -> Icon;
    fn launch(&self);
}
```

### Help Center

Framework and product documentation as a first-class app:

```text
Help Center
  ├─ Nest Framework
  ├─ Nest Shell
  ├─ Kiwi
  ├─ Swift
  ├─ Components
  ├─ CLI Commands
  ├─ App Manifest Format
  └─ Developer Guides
```

### Themes

One Nest theme definition flows through the entire stack:

```text
Nest Theme → Shell → Taskbar → Start Menu → Kiwi → Swift → Settings
```

Shell UI uses [`nest-tailwind-preset.json`](./nest-tailwind-preset.json) and Nest design tokens via CSS variables.

### Native features (via Tauri)

File dialogs, notifications, clipboard, filesystem access, keyboard shortcuts, drag-and-drop, window transparency, optional multi-window, auto-updater, system tray.

## What Nest Shell cannot do

- Manage windows for other OS apps (Chrome, VS Code, etc.).
- Replace the OS desktop or window manager (KDE, GNOME, etc.).
- Control or decorate processes outside the Nest Shell Tauri host.

The desktop exists **inside** one native window, with an in-app window manager rendering Nest application windows only.

## Current status (v0 scaffold)

Early UI scaffold. Hardcoded app list in React; no registry, window manager, or real app embedding yet.

| Area | Status |
|------|--------|
| Desktop background + icon grid | Basic (Help, Kiwi, Swift icons) |
| Start menu | Basic popup launcher |
| Taskbar | Start button, running apps, clock |
| Window manager | Basic (drag, focus, minimize, close) |
| **Help app** | **Docs browser — `docs/` TOC + root README, markdown viewer** |
| App registry / manifests | **`apps/*/nest-app.toml` scanned at startup** |
| Nest backend integration | Partial (`apps_list`, `docs_list`, `docs_read`) |
| Kiwi / Swift embedding | Placeholder windows |
| Notifications | Planned |

Initial launch targets: **Kiwi**, **Swift**, **Settings**, **Terminal** (placeholder).

## Project layout

```text
ui/
├── README.md                 # this file
├── package.json              # React + Vite frontend
├── src/
│   ├── App.tsx               # shell root; app list (temporary)
│   ├── components/
│   │   ├── Desktop.tsx
│   │   ├── StartMenu.tsx
│   │   └── Taskbar.tsx
│   └── lib/
└── src-tauri/                # Tauri 2 host
    ├── src/main.rs
    ├── tauri.conf.json
    └── capabilities/
```

Product apps (Kiwi, Swift, …) live under [`apps/`](../apps/README.md) as separate repositories. Nest Shell will discover and host them via the app registry rather than compiling them into this tree.

## Quick start

```bash
cd ui
npm install
npm run tauri:dev
```

Other commands:

| Command | Description |
|---------|-------------|
| `npm run dev` | Vite dev server only |
| `npm run build` | Production frontend build |
| `npm run tauri:build` | Release Tauri bundle |

Requires Rust toolchain and Linux WebKit/GTK dev packages for Tauri (same as other Nest desktop apps).

## Roadmap (high level)

1. **Window manager** — managed windows, focus, minimize/maximize/close, z-order.
2. **App registry** — manifest format, Rust loader, dynamic Start Menu.
3. **Nest integration** — `nest-tauri` bootstrap, theme sync, logging, config.
4. **Shell apps** — Settings, Terminal placeholder, Help Center (docs reader).
5. **Embed or spawn apps** — load Kiwi/Swift UI in shell windows or controlled child processes.
6. **Polish** — search launcher, notifications, tray, workspaces, file picker via Files app.

## Related

- [Nest architecture](../docs/architecture.md) — layering and desktop frontend platform
- [nest-tauri](../docs/nest-tauri/README.md) — Tauri host crate
- [Desktop template](../templates/desktop/) — minimal single-app Tauri + React starter
- [Kiwi](../apps/kiwi/) — agent IDE / app builder (local checkout)
