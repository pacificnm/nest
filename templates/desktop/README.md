# Nest desktop app template

Starter layout for Nest **desktop** apps: **Tauri + React + TypeScript + Tailwind**.

```text
templates/desktop/
├── ui/                 # React front end (Vite)
├── src-tauri/          # Tauri shell + Nest modules
├── build               # build helper script (mirrors the desktop workflow)
└── README.md           # this file
```

## Features

- [`nest-tauri`](../../core/crates/nest-tauri) bootstrap with `ThemeModule` + `ImageModule`
- Built‑in IPC: you can now call a CLI binary from the UI using the `run_cli` command (see the `src-tauri/src/main.rs` example).
- [`RemoteImage`](ui/src/components/RemoteImage.tsx) React component (cached remote images via Rust)
- Font Awesome icons via `Icon` component.
- Default `cbre-light` theme via `nest-react-theme` CSS variables.
- Shared desktop shell (ribbon, status bar, toasts, etc.).

## Quick start (desktop side)

```bash
# From the template directory (or after scaffolding a new app)
./build dev        # hot‑reload UI + Tauri backend
./build run        # production build + launch
./build build      # produce production artifacts only
```

## Desktop ↔ CLI IPC flow

1. The UI (React) calls `invoke('run_cli', { command: … })`.
2. `src-tauri/src/main.rs` defines a Tauri command `run_cli` that forwards the request to the CLI binary via `nest_tauri::invoke`.
3. The CLI binary (generated from the `templates/cli` scaffold) receives the `CliCommand` enum, runs the requested operation (system command, HTTP GET, etc.), and returns a `String` or a `NestError`.
4. The Tauri command returns the result to the UI, where you can display it in a toast, dialog, or update state.

## Adding new commands

- Extend `cli_command::CliCommand` (shared between desktop and CLI).
- Implement the handling logic in `handle_cli_command` inside the CLI crate.
- Re‑run `cargo build` for the CLI, then the desktop will automatically pick up the new command.

## Scaffolding a new desktop app

```bash
scripts/scaffold-desktop-app.sh apps/<name> "Display Title"
cd apps/<name>
./build dev
```

## Related

- [nest-tauri docs](../../docs/nest-tauri/README.md)
- [nest-react-ui v1 plan](../../docs/plan/nest-react-ui-v1.md)

