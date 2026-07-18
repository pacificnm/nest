# Nest Framework

Nest is a modular application framework for Rust. Applications opt into small, focused crates instead of inheriting a monolithic stack. This repository contains the **framework** (`core/`) and optional **integration modules** (`modules/`). Shipping products live in **separate repositories**.

> **Only compile and load the functionality your application actually needs.**

## Repository layout

```text
nest/
├── Cargo.toml              # Workspace
├── README.md
├── docs/
├── examples/
├── core/crates/            # Framework (stable, reviewed)
│   ├── nest-core, nest-app, nest-cli, nest-tui, nest-tauri
│   ├── nest-config, nest-error, nest-logging
│   ├── nest-task, nest-task-runtime
│   ├── nest-file, nest-file-csv
│   ├── nest-http, nest-http-client, nest-http-serve, nest-media
│   ├── nest-data, nest-validation, nest-design, nest-theme
│   └── ...
├── modules/crates/         # Optional integrations
│   ├── nest-airtable
│   ├── nest-data-sqlite
│   ├── nest-media-library
│   ├── nest-tmdb
│   ├── nest-transcode
│   └── nest-github, nest-postgres, ... (planned)
├── apps/                   # Local product clones (gitignored — see apps/README.md)
└── tools/
```

**Rule:** `core/` holds the framework contract. `modules/` holds optional adapters. **`apps/` is for local product checkouts only** — ignored by nest git, never committed.

See [docs/architecture.md](docs/architecture.md) for layering and dependency rules. Product repos follow [docs/app-standard.md](docs/app-standard.md).

## Layering

```text
Apps  →  Modules  →  Core
```

| Layer | Depends on | Must not depend on |
|-------|------------|-------------------|
| **Core** (`core/crates/`) | Core only | Modules, apps |
| **Modules** (`modules/crates/`) | Core | Apps |
| **Apps** (separate repos) | Core, modules | — |

This scales as the workspace grows: contributors can tell at a glance where new functionality belongs.

## Runtime stack (core)

```text
nest-core          primitives (AppBuilder, Module, services, lifecycle)
    ↓
nest-app           host-agnostic container (metadata, startup/shutdown)
    ↓
nest-cli / nest-tui / nest-tauri   presentation hosts
```

Modules (`nest-airtable`, `nest-data-sqlite`, …) plug into the container; apps compose hosts + modules.

Hosts own CLI parsing, event loops, logging initialization, and config file loading.

**Build commands:** every app uses `./build` with the same verbs (`build`, `run`, `dev`, `test`, `check`, `clean`). See [docs/build.md](docs/build.md) and [docs/app-standard.md](docs/app-standard.md).

**Nest Shell:** from the repo root, run `./start` to launch the desktop shell demo ([`ui/`](ui/)) in development mode (Tauri + Vite + embed app dev servers).

## Desktop frontend platform

Nest **desktop** apps use **Tauri + React + TypeScript + Tailwind**:

| Layer | Path |
|-------|------|
| Tauri shell + Nest modules | `src-tauri/` |
| React UI + Tailwind | `ui/` |

Rust business logic stays in Nest modules; React is the presentation tier. See [docs/architecture.md](docs/architecture.md#desktop-frontend-platform).

---

## Core crates (`core/crates/`)

### Application

| Crate | Summary | Docs |
|-------|---------|------|
| **nest-core** | Module system, `AppBuilder`, service registry, `AppContext`, and synchronous lifecycle hooks. | [docs](docs/nest-core/README.md) |
| **nest-app** | Wraps `nest-core` with `NestApp`, metadata, bootstrap validation, and traced startup/shutdown. | [docs](docs/nest-app/README.md) |

### Hosts

| Crate | Summary | Docs |
|-------|---------|------|
| **nest-cli** | Command-line host: clap parsing, config/logging init, command dispatch, exit codes. | [docs](docs/nest-cli/README.md) |
| **nest-tui** | Terminal UI host: Ratatui event loop, file-only logging, config merge. | [docs](docs/nest-tui/README.md) |
| **nest-tauri** | Desktop host: Tauri shell, IPC, `ui/` + React + Tailwind frontend. | [docs](docs/nest-tauri/README.md) |

### Foundation

| Crate | Summary | Docs |
|-------|---------|------|
| **nest-error** | Shared `NestError` model, stable `NEST_*` codes, and UI-ready `NestErrorReport`. | [docs](docs/nest-error/README.md) |
| **nest-config** | TOML/JSON config loading, search paths, and `ConfigService` for section access. | [docs](docs/nest-config/README.md) |
| **nest-logging** | Tracing-based logging for hosts: console, file, rotation, module filters. | [docs](docs/nest-logging/README.md) |

### Data contracts

| Crate | Summary | Docs |
|-------|---------|------|
| **nest-data** | Database-agnostic contracts: repositories, transactions, migrations, connection lifecycle. | [docs](docs/nest-data/README.md) |

### HTTP

| Crate | Summary | Docs |
|-------|---------|------|
| **nest-http** | Shared HTTP types: methods, status, headers, request/response, auth/retry contracts. | [docs](docs/nest-http/README.md) |
| **nest-http-client** | Async reqwest client behind `HttpClientService` and `HttpClientModule`. | [docs](docs/nest-http-client/README.md) |
| **nest-http-serve** | Reusable HTTP host: routing, JSON, static files, SPA fallback, CORS. | [docs](docs/nest-http-serve/README.md) |
| **nest-media** | Media domain models and provider traits (movies v0.1). | [docs](docs/nest-media/README.md) |

### Tasks

| Crate | Summary | Docs |
|-------|---------|------|
| **nest-task** | Task contracts: handles, progress, cancellation, and events (no runtime). | [docs](docs/nest-task/README.md) |
| **nest-task-runtime** | Tokio-backed `TaskRuntime` and `TaskManagerService` for scheduling `nest-task` work. | [docs](docs/nest-task-runtime/README.md) |

### Files

| Crate | Summary | Docs |
|-------|---------|------|
| **nest-file** | Scoped sync file I/O via `FileService` and `FileModule`. | [docs](docs/nest-file/README.md) |
| **nest-file-csv** | CSV import/export: parsing, mapping, validation, and reporting over `nest-file`. | [docs](docs/nest-file-csv/README.md) |

### Design & theme

| Crate | Summary | Docs |
|-------|---------|------|
| **nest-design** | Design token schema and built-in theme definitions (no runtime or UI deps). | [docs](docs/nest-design/README.md) |
| **nest-theme** | Runtime theme loading, validation, and `ThemeService` lifecycle. | [docs](docs/nest-theme/README.md) |
| **nest-react-theme** | Theme tokens → CSS variables + Tailwind preset for React `ui/`. | [docs](docs/nest-react-theme/README.md) |

### Validation

| Crate | Summary | Docs |
|-------|---------|------|
| **nest-validation** | UI-agnostic validators, `Validate` trait, and `ValidatorRegistry` via `ValidationModule`. | [docs](docs/nest-validation/README.md) |

---

## Integration modules (`modules/crates/`)

| Crate | Summary | Docs |
|-------|---------|------|
| **nest-airtable** | Airtable REST client: offset pagination, batch updates, Bearer auth, rate-limit retry. | [docs](docs/nest-airtable/README.md) |
| **nest-claude** | Claude (Anthropic) Messages API client: streaming, tool use, extended thinking, prompt caching. | [docs](docs/nest-claude/README.md) |
| **nest-data-sqlite** | SQLite provider implementing `nest-data` via rusqlite. | [docs](docs/nest-data-sqlite/README.md) |
| **nest-media-library** | Media library scanning and indexing via `FileService` and injected providers. | [docs](docs/nest-media-library/README.md) |
| **nest-mqtt** | MQTT client: connect, publish, subscribe, Last-Will-and-Testament, via `rumqttc`. | [docs](docs/nest-mqtt/README.md) |
| **nest-tmdb** | TMDB metadata provider implementing `nest-media::MetadataProvider`. | [docs](docs/nest-tmdb/README.md) |
| **nest-transcode** | FFprobe media inspection implementing `nest-media::MediaInspector`. | [docs](docs/nest-transcode/README.md) |

Planned: `nest-github`, `nest-git`, `nest-postgres`, `nest-docker`, `nest-kubernetes`, `nest-transcode`, …

---

## Applications (`apps/`)

Products live in **separate repositories**. For local Pacific NM development, clone them into `apps/<name>/` — nest git ignores those directories. See [apps/README.md](apps/README.md).

| Local path (gitignored) | Repository | Summary |
|-------------------------|------------|---------|
| `apps/airtable-sync/` | [pacificnm/airtable-sync](https://github.com/pacificnm/airtable-sync) | Airtable sync CLI (`tables`, `list`) via `nest-airtable`. |

Planned: `kiwi`, `finch`, …

---

## Quick start (desktop)

Nest desktop apps use **Tauri + React + TypeScript + Tailwind**. Rust modules run in `src-tauri/`; UI lives in `ui/`.

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

```tsx
// ui/src/App.tsx
export function App() {
  return <h1 className="text-nest-foreground">Hello, Nest</h1>;
}
```

See [nest-tauri](docs/nest-tauri/README.md) and [architecture — desktop frontend platform](docs/architecture.md#desktop-frontend-platform).

Workspace dependencies use `{ workspace = true }` — paths are defined in the root [`Cargo.toml`](Cargo.toml).

**First run:** use `./build dev` (see [docs/build.md](docs/build.md)), not
`npm run dev` directly — it also installs dependencies for any locally
path-referenced `@nest/*` package (e.g. `@nest/components`), which plain
`npm install` in `ui/` does not do and which otherwise fails silently with
a confusing Vite module-resolution error.

---

## Development / agent tools

Local MCP servers (Cursor) provide semantic search over project docs, reference manuals (Rust, Tauri, React, Tailwind), and persistent agent context:

- [`tools/MCP-SETUP.md`](tools/MCP-SETUP.md) — PostgreSQL, Python venv, Cursor MCP, hooks
- [`AGENTS.md`](AGENTS.md) — agent workflow and memory usage

Quick start after PostgreSQL + `nest_memory` database:

```bash
python3 -m venv .venv && .venv/bin/pip install -r tools/requirements.txt
cp .env.example .env   # set DATABASE_URL, OLLAMA_HOST, OLLAMA_EMBED_MODEL
./scripts/setup-memory.sh
./scripts/index-memory.sh
./scripts/index-knowledge.sh   # requires ~/nest-knowledge
```
