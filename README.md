# Nest Framework

Nest is a modular application framework for Rust. Applications opt into small, focused crates instead of inheriting a monolithic stack. The core defines contracts (modules, services, lifecycle); optional crates add configuration, data access, validation, file I/O, HTTP, tasks, and presentation hosts.

> **Only compile and load the functionality your application actually needs.**

## Architecture

```text
nest-core          primitives (AppBuilder, Module, services, lifecycle)
    ↓
nest-app           host-agnostic container (metadata, startup/shutdown)
    ↓
nest-cli / nest-tui / nest-gui   presentation hosts
```

Hosts own CLI parsing, event loops, logging initialization, and config file loading. Feature crates register services via `nest-core` modules.

---

## Modules

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
| **nest-gui** | Desktop GUI host: eframe/egui main loop, window options, file-only logging. | [docs](docs/nest-gui/README.md) |

### Foundation

| Crate | Summary | Docs |
|-------|---------|------|
| **nest-error** | Shared `NestError` model, stable `NEST_*` codes, and UI-ready `NestErrorReport`. | [docs](docs/nest-error/README.md) |
| **nest-config** | TOML/JSON config loading, search paths, and `ConfigService` for section access. | [docs](docs/nest-config/README.md) |
| **nest-logging** | Tracing-based logging for hosts: console, file, rotation, module filters. | [docs](docs/nest-logging/README.md) |

### Data

| Crate | Summary | Docs |
|-------|---------|------|
| **nest-data** | Database-agnostic contracts: repositories, transactions, migrations, connection lifecycle. | [docs](docs/nest-data/README.md) |
| **nest-data-sqlite** | SQLite provider implementing `nest-data` via rusqlite. | [docs](docs/nest-data-sqlite/README.md) |

### HTTP

| Crate | Summary | Docs |
|-------|---------|------|
| **nest-http** | Shared HTTP types: methods, status, headers, request/response, auth/retry contracts. | [docs](docs/nest-http/README.md) |
| **nest-http-client** | Async reqwest client behind `HttpClientService` and `HttpClientModule`. | [docs](docs/nest-http-client/README.md) |
| **nest-airtable** | Airtable REST client: offset pagination, batch updates, Bearer auth, rate-limit retry. | [docs](docs/nest-airtable/README.md) |

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

### Validation

| Crate | Summary | Docs |
|-------|---------|------|
| **nest-validation** | UI-agnostic validators, `Validate` trait, and `ValidatorRegistry` via `ValidationModule`. | [docs](docs/nest-validation/README.md) |

---

## Quick start

```rust
use nest_gui::{GuiApp, GuiView};
use nest_core::AppContext;
use nest_error::NestResult;
use nest_theme::ThemeModule;

struct MainView;

impl GuiView for MainView {
    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &AppContext) -> NestResult<()> {
        ui.heading("Hello, Nest");
        Ok(())
    }
}

fn main() {
    GuiApp::new("my-app")
        .module(ThemeModule::default())
        .view(MainView)
        .run();
}
```

For a pre-built container and shared modules across hosts, see [nest-app](docs/nest-app/README.md).

---

## Development / agent tools

Local MCP servers (Cursor) provide semantic search over project docs, reference manuals (Rust, egui), and persistent agent context:

- [`tools/MCP-SETUP.md`](tools/MCP-SETUP.md) — PostgreSQL, Python venv, Cursor MCP, hooks
- [`AGENTS.md`](AGENTS.md) — agent workflow and memory usage

Quick start after PostgreSQL + `nest_memory` database:

```bash
python3 -m venv .venv && .venv/bin/pip install -r tools/requirements.txt
cp .env.example .env   # set OPENAI_API_KEY
./scripts/setup-memory.sh
./scripts/index-memory.sh
./scripts/index-knowledge.sh   # requires ~/nest-knowledge
```
