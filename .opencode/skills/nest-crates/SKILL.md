---
name: nest-crates
description: Use when the user needs a Nest shared crate (core/crates/ or modules/crates/) for a feature in apps/ — error handling, config, HTTP client/server, logging, caching, tasks, Tauri, TUI, CLI, file I/O, images, theming, AI/agent/MCP integration — or asks "which crate should I use for...".
---

# Nest Crates Skill

**Purpose**: Help developers use Nest shared crates from `core/crates/` and `modules/crates/` when building applications in `apps/`.

## When to use

- User is adding a new feature to an app in `apps/`
- User asks about which Nest crate to use for a specific need
- User needs to understand crate dependencies and APIs
- User is setting up a new app and needs to configure workspace dependencies

## Crate Catalog

### Core Crates (`core/crates/`)

| Crate | Purpose | Common Use Cases |
|-------|---------|------------------|
| `nest-core` | Framework contracts, `AppBuilder`, `Module` trait, module system | Building custom hosts, module registration |
| `nest-app` | `NestApp` application container, shared bootstrap | Host lifecycle wiring |
| `nest-error` | Shared error types (`NestError`, `NestResult`) | All error handling |
| `nest-config` | Configuration loading (`ConfigService`, `ConfigLoader`, `ConfigDocument`) | App configuration, settings |
| `nest-data` | Database-agnostic data contracts | Repository pattern, data access |
| `nest-http` | HTTP contracts | API request/response types |
| `nest-http-client` | HTTP client (`HttpClientService`) | External API calls |
| `nest-http-serve` | HTTP server | REST APIs, webhooks |
| `nest-logging` | Logging setup (`tracing`-based, `init(LoggingConfig)`) | Application logging |
| `nest-validation` | Input validation (`Validate` trait, `validate()`) | Form validation, data validation |
| `nest-cache` | Caching (`Cache`, `CacheAdapter`) | Performance optimization |
| `nest-task` / `nest-task-runtime` | Background task system (`Task` trait, `TaskManager`) | Async operations, job queues |
| `nest-tauri` | Tauri desktop host (`TauriApp`) | Desktop apps with Tauri + React |
| `nest-tui` | Terminal UI host | CLI apps with TUI |
| `nest-cli` | CLI host | Command-line applications |
| `nest-file` | File operations | File I/O abstractions |
| `nest-file-csv` | CSV file handling | CSV import/export |
| `nest-media` | Media processing | Image/video handling |
| `nest-image` | Image service | Remote images, caching |
| `nest-design` / `nest-theme` / `nest-react-theme` / `nest-react-components` | Design tokens, theming, shared React components | UI consistency across apps |
| `nest-ai` | AI provider abstraction | LLM calls, AI features |
| `nest-agent` | Agent system | Autonomous agents |
| `nest-mcp` | MCP (Model Context Protocol) | AI context management |
| `nest-stream` | Streaming abstractions | Data streams |

### Module Crates (`modules/crates/`)

Adapters and integrations that wrap external systems — see
[nest.md](../instructions/nest.md) for the layering
rule (modules depend on core only, never on apps or other modules unless
necessary).

| Crate | Purpose |
|-------|---------|
| `nest-airtable` | Airtable API integration |
| `nest-tmdb` | TMDB (movie/TV metadata) integration |
| `nest-transcode` | Media transcoding |
| `nest-mqtt` | MQTT client (`rumqttc`-backed) |
| `nest-claude` | Claude API client |
| `nest-ai-claude` / `nest-ai-ollama` | `AiProvider` adapters for `nest-ai` |
| `nest-data-postgres` / `nest-data-sqlite` | `nest-data` backend adapters |
| `nest-cache-file` | File-backed `CacheAdapter` |
| `nest-media-library` | Media library scanning/organization |

### Dependency Rules

1. **Apps** → may depend on any core/module crate
2. **Modules** → may depend on core crates only (avoid depending on other modules unless necessary)
3. **Core** → may depend on other core crates only
4. Core/Modules must NEVER depend on apps

Full detail: [nest.md](../instructions/nest.md), [docs/architecture.md](../../docs/architecture.md).

## Common Patterns

Verify exact method names against the crate's real source before writing
code from memory — the examples below are checked against current source,
but APIs do change.

### Adding a Nest crate dependency to an app

```toml
# In app's Cargo.toml
[dependencies]
nest-error = { workspace = true }
nest-config = { workspace = true }
```

### Using NestError for error handling

```rust
use nest_error::{NestError, NestResult};

fn do_something() -> NestResult<MyType> {
    // Return Ok(...) or Err(NestError::validation("...").with_code("NEST_..."))
}
```

See [nest.md](../instructions/nest.md) for the full
pattern (codes, `.with_help`, `NestResultExt::nest_context`, and the
`clippy::result_large_err` gotcha).

### Loading configuration

```rust
use nest_config::{ConfigLoader, ConfigService};

let loaded = ConfigLoader::new("my-app").load()?; // searches default paths
let config_service = ConfigService::new(loaded);
let section: MyAppConfig = config_service.section("my_app")?;
```

There is no `Config::load(path)` convenience constructor — configuration is
loaded via `ConfigLoader`, wrapped in a `ConfigService`, and individual TOML
sections are deserialized with `.section("name")`.

### Setting up logging

```rust
use nest_logging::prelude::*;

init(
    LoggingConfig::new("my-app")
        .with_console()
        .with_default_level(LogLevel::Info),
)?;

tracing::info!(target: "my_app", "application started");
```

`init` requires a `LoggingConfig` argument — there is no zero-argument
`init()`. See [nest.md](../instructions/nest.md).

### Building a Tauri desktop app

```rust
// In src-tauri/src/main.rs
use nest_tauri::TauriApp;

fn main() {
    TauriApp::new("my-app")
        .module(MyModule)
        .run(tauri::generate_context!());
}
```

The bootstrap type is `TauriApp` (`TauriApp::new(name).module(m).run(ctx)`),
not a `TauriHost::builder()` pattern. See [nest.md](../instructions/nest.md).

## Workspace Setup

When an app is checked out locally under `apps/<name>/` alongside this
framework repo, reference Nest crates by path in the app's own
`[workspace.dependencies]` (see e.g. `apps/sparrow/Cargo.toml` for a real
example of this pattern):

```toml
# In the app's own Cargo.toml
[workspace.dependencies]
nest-core = { path = "../../core/crates/nest-core" }
nest-error = { path = "../../core/crates/nest-error" }
```

When the app is developed outside this monorepo layout (its own standalone
repo, not checked out under `apps/`), it depends on Nest crates via `git`:

```toml
[dependencies]
nest-core = { git = "https://github.com/pacificnm/nest" }
nest-error = { git = "https://github.com/pacificnm/nest" }
```

See [apps/README.md](../../apps/README.md) for the authoritative layout and
patching conventions.

## Questions to Answer

1. Which Nest crate should I use for [X]?
2. How do I add [nest-*] crate to my app?
3. What's the dependency chain for [nest-*]?
4. Show me examples of using [nest-*] crate
5. How do Nest crates follow the architecture layers?

## Related Files

- [docs/architecture.md](../../docs/architecture.md) — layer architecture
- `core/crates/` — all core crate sources
- `modules/crates/` — all module sources
- [apps/README.md](../../apps/README.md) — app development guide
