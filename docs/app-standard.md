# Nest app standard

Every Nest **product** follows one runtime model: **one Rust core, host adapters for presentation, IPC only at the desktop webview boundary.**

This document is the canonical reference for how apps are structured, how hosts reach the core, and how builds stay consistent. For repository layering (core / modules / apps), see [architecture.md](architecture.md). For `./build` commands, see [build.md](build.md).

## Principles

1. **Business logic lives in Rust** — product domain code and Nest modules; not in React, not duplicated in CLI/TUI shells.
2. **Hosts are presentation only** — `nest-cli`, `nest-tui`, and `nest-tauri` parse input, render output, and wire calls into `AppContext`.
3. **IPC is desktop-only** — React runs in a webview; it reaches Rust via Tauri `invoke` / events. CLI and TUI are in-process Rust and call services directly.
4. **One command surface, multiple adapters** — implement an operation once in `crates/core` (or a product service); expose it through the host that fits (subcommand, screen action, or `#[tauri::command]`).
5. **Same bootstrap everywhere** — config, logging, `AppContext`, modules; hosts differ only after the container is ready.

## Runtime stack

```text
┌─────────────────────────────────────────────────────────────┐
│  Product Rust core (crates/core, app services)              │
│  + Nest modules (nest-file, nest-data, nest-http, …)        │
│  → AppContext / registered services                         │
└───────────────────────────┬─────────────────────────────────┘
                            │ in-process calls
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
   nest-cli            nest-tui            nest-tauri
   CliCommand          TuiScreen           #[tauri::command]
        │                   │                   │
   stdout/stderr      Ratatui loop            │ Tauri IPC
                                               ▼
                                          ui/ React + Tailwind
```

Framework crates:

```text
nest-core → nest-app → nest-cli | nest-tui | nest-tauri
```

## Host matrix

| Surface | Host crate | Presentation | Reaches core via | Typical folder |
|---------|------------|--------------|------------------|----------------|
| **Desktop GUI** | `nest-tauri` | React + TypeScript + Tailwind | Tauri IPC → command handlers in `src-tauri/` | `ui/` + `src-tauri/` |
| **CLI** | `nest-cli` | Subcommands, flags, stdout/stderr | Direct Rust — `CliCommand::run(&AppContext)` | `crates/cli` (+ `crates/core`) |
| **TUI** | `nest-tui` | Ratatui screens, keyboard/mouse | Direct Rust — `TuiScreen` methods receive `&AppContext` | `crates/tui` (+ `crates/core`) |
| **Server** | `nest-http-serve` (or custom binary) | HTTP routes | Direct Rust — handlers use `AppContext` / services | `crates/server` (+ `crates/core`) |

A product may ship **one or more** surfaces (e.g. Loon: HTTP server + desktop admin + webOS client). Each surface gets its own host; all share the same Rust core where possible.

### Desktop frontend platform

All Nest **desktop** apps use the same frontend stack:

| Layer | Technology | Location |
|-------|------------|----------|
| Shell | **Tauri** | `src-tauri/`, `nest-tauri` |
| UI | **React** + **TypeScript** | `ui/src/` (Vite) |
| Styling | **Tailwind CSS** | `ui/tailwind.config.ts`, `nest-react-theme` |
| Icons | React icon library | `ui/` (e.g. Font Awesome, Lucide) |
| Remote images | `nest-image` + React | Rust `ImageService` + `<RemoteImage>` in `ui/` |
| Tokens | `nest-design` / `nest-theme` | Rust → CSS variables in webview |

React is **presentation only**. It calls Tauri commands and listens for events; it does not replace Nest modules.

## IPC boundary (desktop only)

```text
ui/src/          invoke("scan_library", { … })
       ───────── Tauri IPC (serialize args / results) ─────────
src-tauri/       #[tauri::command] fn scan_library(ctx: State<…>, …)
       ───────── in-process ─────────
crates/core/     pub async fn scan_library(svc: &LibraryService, …) -> NestResult<…>
```

Rules:

- **Do** keep IPC handlers thin: validate input, resolve services from `AppContext`, delegate to `crates/core`.
- **Do** use structured errors (`NestError` / `NestErrorReport`) bridged to the webview.
- **Do not** put business rules in `ui/` beyond form validation and display logic.
- **Do not** use IPC between CLI/TUI and core — they are already in the same process.

Built-in Tauri commands from `nest-tauri` (when `runtime` feature is enabled): `nest_app_metadata`, `nest_theme_css`, `nest_image_fetch`, `nest_image_invalidate_tag`. See [nest-tauri README](nest-tauri/README.md).

## Shared bootstrap

All hosts follow the same order (see [nest-app](nest-app/README.md)):

1. Parse startup options (CLI flags, Tauri CLI, TUI flags)
2. Load config (`nest-config`) — precedence: **defaults < config file < CLI flags**
3. Initialize logging (`nest-logging`) — desktop and TUI default to **file-only** while the UI is active
4. Build `AppContext` via `AppBuilder` / `NestApp`
5. Register modules and services
6. Run the host main loop (command, terminal loop, or Tauri + webview)
7. Shutdown and restore (terminal, windows, lifecycle hooks)

## Command surface pattern

Implement product operations **once** in Rust, then wire each host:

```text
crates/core/src/commands/scan_library.rs   ← shared handler
        ↑                    ↑                    ↑
crates/cli/…                 crates/tui/…         src-tauri/commands/…
CliCommand                   TuiScreen::on_event  #[tauri::command]
```

Example shape (conceptual):

```rust
// crates/core — domain logic
pub async fn scan_library(ctx: &AppContext) -> NestResult<ScanReport> {
    let svc = ctx.get::<LibraryService>()?;
    svc.scan().await
}

// crates/cli — host adapter
impl CliCommand for ScanCmd {
    fn run(&self, ctx: &AppContext) -> NestResult<()> {
        let report = scan_library(ctx).block_on()?;
        println!("{}", report);
        Ok(())
    }
}

// src-tauri — host adapter (IPC)
#[tauri::command]
async fn scan_library_cmd(state: State<'_, AppState>) -> Result<ScanReport, String> {
    scan_library(state.context()).await.map_err(|e| e.to_string())
}
```

Keep handlers free of presentation concerns (no `println!` in core, no Ratatui types in core, no React types in Rust).

## Folder layouts

### Desktop (Tauri + React)

Template: [`templates/desktop/`](../templates/desktop/)

```text
my-app/
├── build                 # NEST_BUILD_PROFILE=tauri
├── config.toml
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
└── crates/
    └── core/             # shared product logic (recommended)
```

### CLI

Template: [`templates/rust-server/build`](../templates/rust-server/build) (build wrapper; add crates as needed)

```text
my-app/
├── build                 # NEST_BUILD_PROFILE=rust
├── config.toml
├── crates/
│   ├── core/             # domain logic + services
│   └── cli/
│       └── src/
│           └── main.rs   # CliApp::new(…).command(…).run()
└── Cargo.toml            # workspace
```

### TUI (Ratatui)

```text
my-app/
├── build                 # NEST_BUILD_PROFILE=rust
├── config.toml
├── crates/
│   ├── core/
│   └── tui/
│       └── src/
│           ├── main.rs   # TuiApp::new(…).screen(…).run()
│           └── screens/
└── Cargo.toml
```

### Multi-surface product

```text
loon/                     # example layout
├── server/               # HTTP API (rust profile)
├── desktop/              # Tauri admin (tauri profile): ui/ + src-tauri/
├── client/               # webOS / Vite-only (node profile) — not Nest Tauri
└── crates/core/          # shared Rust logic for server + desktop
```

Use one `./build` per surface; each sets `NEST_BUILD_PROFILE` appropriately.

## What goes where

| You are building… | Put it in… |
|-------------------|------------|
| Domain rules, workflows, orchestration | `crates/core` (product repo) |
| Third-party integration (DB, API) | Nest **module** in framework repo, or thin adapter in product |
| CLI subcommands, flag parsing | `crates/cli` — delegate to core |
| Terminal layout, keybindings | `crates/tui` — delegate to core |
| Tauri command registration | `src-tauri/src/commands/` — delegate to core |
| Pages, components, styling | `ui/src/` — call Tauri IPC only |
| Module registration, config wiring | Host `main.rs` (`CliApp`, `TuiApp`, `TauriApp`) |

## Build interface

Every product repo exposes the same **`./build`** verbs:

| Command | Meaning |
|---------|---------|
| `./build` / `./build build` | Production build |
| `./build run` | Build if needed, launch |
| `./build dev` | Development (hot reload / `cargo run` / `tauri dev`) |
| `./build test` | Tests |
| `./build check` | CI checks |
| `./build clean` | Remove artifacts |

Profiles: `rust`, `tauri`, `node`, `workspace`. See [build.md](build.md).

## Legacy (do not use for new work)

| Old | Use instead |
|-----|-------------|
| `nest-gui` (egui) | `nest-tauri` + React `ui/` |
| `nest-icon` (egui) | React icons in `ui/` |
| `nest-image` egui widget | `ImageService` + `<RemoteImage>` |
| Business logic in React | Rust `crates/core` + Tauri commands |

## Related

- [architecture.md](architecture.md) — framework layering and dependency rules
- [build.md](build.md) — `./build` commands and profiles
- [apps/README.md](../apps/README.md) — local product checkout layout
- [nest-app](nest-app/README.md) — `NestApp`, lifecycle, host handoff
- [nest-cli](nest-cli/README.md) · [nest-tui](nest-tui/README.md) · [nest-tauri](nest-tauri/README.md)
- [nest-tauri v1 plan](plan/nest-tauri-v1.md) · [nest-react-ui v1 plan](plan/nest-react-ui-v1.md)
- [templates/desktop/](../templates/desktop/) — desktop starter
