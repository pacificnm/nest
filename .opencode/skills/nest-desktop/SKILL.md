---
name: nest-desktop
sdk:
  version: 0.1.0
description: |-
  Scaffold for a Nest desktop application using Tauri, React, TypeScript and Tailwind.
  Provides the UI shell, IPC hooks, and build helpers.
---
# nest‑desktop Template Skill

## What it offers
The template provides
- A **Tauri + React** starter using Vite, TypeScript and Tailwind.
- Shared UI shell components (`AppShell`, `Ribbon`, `StatusBar`, etc.) in `ui/src/components`.
- Built‑in IPC commands (e.g., `nest_app_metadata`, `nest_theme_css`) via `src-tauri`.
- A cached remote image component (`RemoteImage.tsx`).
- Tailwind preset aligned with the official [`nest-react-theme`](../../core/crates/nest-react-theme).

## Quick start
From the template directory:

```bash
./build dev   # hot reload for Tauri + Vite
./build run   # production build + launch
./build build # only artifacts
```

See detailed commands in [Nest build standard](../../docs/build.md).

## Core files

- `ui/` – React front‑end with Vite, TS and Tailwind. The directory contains:
  - `src/components/`: layout shell (`AppShell`, `Ribbon`, `StatusBar`, toast components).
  - `src/main.tsx` – mounts the root UI.
  - `src/lib/date.ts`, `iconfont…` – utilities used by the shell.
- `src-tauri/` – Tauri shell and Rust side. Includes IPC commands (`nest_app_metadata`, `nest_theme_css`, …) and Nest module setup via `nest-tauri`.
- `build` – helper scripts for dev, prod build and run. See README for usage.
- `ui/` – React front‑end (Vite config, components, shell, etc.).
- `src-tauri/` – Tauri shell with Nest modules and IPC.

## Template location

The scaffold lives in the repository under:

```
templates/desktop/
```

Don't copy this folder by hand. Use `scripts/scaffold-desktop-app.sh`:

```bash
scripts/scaffold-desktop-app.sh apps/<name> "Display Title"
cd apps/<name>
./build dev
```

It copies `ui/`, `src-tauri/`, `build`, `nest-app.toml`, and `.gitignore`,
and renames every placeholder (Cargo package name, Tauri bundle identifier,
window title, UI package name, cache dir) to match `<name>`. See the
"Scaffolding a new app" section in **README.md** for details.

---
*References*
- Template README: `README.md`
- App shell component: `ui/src/components/AppShell.tsx`