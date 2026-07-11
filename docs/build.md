# Nest build standard

Every Nest app uses the same **`./build`** interface. Same verbs, same meaning — Rust server, Tauri desktop, or React-only.

For runtime architecture (Rust core, hosts, IPC, folder layouts), see [app-standard.md](app-standard.md).

## Commands

| Command | What it does |
|---------|----------------|
| `./build` | Production build (**default**) |
| `./build build` | Same as above |
| `./build run` | Build if needed, then **launch** the app |
| `./build dev` | **Development** mode (hot reload / `cargo run`) |
| `./build test` | Run tests |
| `./build check` | CI checks (fmt, clippy, tests, UI build) |
| `./build clean` | Remove `target/`, `ui/dist/`, etc. |

Pass extra args after the command. For `run`, use `--` before app args:

```bash
./build run --release -- -- --config config.toml
```

## By app type

### Rust server / CLI (`NEST_BUILD_PROFILE=rust`)

```bash
./build build    # cargo build --release
./build run      # launch target/release/<binary>
./build dev      # cargo run
./build test     # cargo test
```

**Example:** `apps/loon` (server)

```bash
cd apps/loon
./build build
./build run --release
```

### Tauri desktop (`NEST_BUILD_PROFILE=tauri`)

```bash
./build build    # npm run build + release Rust binary (or tauri bundle)
./build run      # build + launch desktop app
./build dev      # tauri dev (Vite + webview)
./build release  # full Tauri bundle (.deb, etc.) when NEST_TAURI_MODE=bundle
```

**Example:** Loon Admin desktop

```bash
cd apps/loon/desktop
./build dev      # daily development
./build run      # production binary
```

### React / Vite only (`NEST_BUILD_PROFILE=node`)

```bash
./build dev      # vite dev server
./build build    # production dist/
./build test     # npm test
```

**Example:** Loon webOS client

```bash
cd apps/loon/client
./build dev
```

### Nest framework workspace (`NEST_BUILD_PROFILE=workspace`)

From the **nest repo root**:

```bash
./build build    # cargo build --workspace
./build test     # cargo test --workspace
./build check    # fmt + clippy + test
./start          # Nest Shell desktop demo (ui/) — Tauri dev
```

## Implementation

Shared logic lives in [`scripts/nest-build/lib.sh`](../scripts/nest-build/lib.sh).

Each app’s `./build` is a thin wrapper that sets:

| Variable | Purpose |
|----------|---------|
| `NEST_BUILD_PROFILE` | `rust`, `tauri`, `node`, or `workspace` |
| `NEST_RUST_PACKAGE` | Cargo package name |
| `NEST_RUST_BIN` | Binary name for `./build run` |
| `NEST_UI_DIR` | Front-end folder (default `ui`) |
| `NEST_CARGO_MANIFEST` | Optional path to `Cargo.toml` |
| `NEST_TAURI_MODE` | `binary` (default) or `bundle` |
| `NEST_CONFIG_FILE` | Config passed to `./build run` |

Templates:

- [`templates/desktop/build`](../templates/desktop/build) — Tauri + React
- [`templates/rust-server/build`](../templates/rust-server/build) — Rust server/CLI

## Cheat sheet

```text
I want to…                    Run
────────────────────────────────────────────────────
Develop Tauri desktop         ./build dev
Run desktop release build     ./build run
Build server for deploy       ./build build
Run server locally            ./build run --release
Run framework tests           cd nest && ./build test
Clean everything              ./build clean
```

## Related

- [app-standard.md](app-standard.md) — product runtime model and folder layouts
- [apps/README.md](../apps/README.md) — product checkout layout
- [architecture.md](architecture.md) — framework layering (core / modules / apps)
