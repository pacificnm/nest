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
| `nest-core` | Framework contracts, `AppBuilder`, module system | Building custom hosts, module registration |
| `nest-app` | Application container | App lifecycle, service registry |
| `nest-error` | Shared error types (`NestError`, `NestResult`) | All error handling |
| `nest-config` | Configuration loading (TOML, JSON) | App configuration, settings |
| `nest-data` | Database-agnostic data contracts | Repository pattern, data access |
| `nest-http` | HTTP contracts | API request/response types |
| `nest-http-client` | HTTP client wrapper | External API calls |
| `nest-http-serve` | HTTP server | REST APIs, webhooks |
| `nest-logging` | Logging setup (tracing) | Application logging |
| `nest-validation` | Input validation | Form validation, data validation |
| `nest-cache` | Caching abstractions | Performance optimization |
| `nest-task` / `nest-task-runtime` | Background task system | Async operations, job queues |
| `nest-tauri` | Tauri desktop host | Desktop apps with Tauri + React |
| `nest-tui` | Terminal UI host | CLI apps with TUI |
| `nest-cli` | CLI host | Command-line applications |
| `nest-file` | File operations | File I/O abstractions |
| `nest-file-csv` | CSV file handling | CSV import/export |
| `nest-media` | Media processing | Image/video handling |
| `nest-image` | Image service | Remote images, caching |
| `nest-design` / `nest-theme` / `nest-react-theme` | Design tokens, theming | UI consistency across apps |
| `nest-gui` | GUI abstractions | Cross-platform UI |
| `nest-icon` | Icon service | Application icons |
| `nest-ai` | AI integrations | LLM calls, AI features |
| `nest-agent` | Agent system | Autonomous agents |
| `nest-mcp` | MCP (Model Context Protocol) | AI context management |
| `nest-stream` | Streaming abstractions | Data streams |

### Dependency Rules

1. **Apps** → may depend on any core/module crate
2. **Modules** → may depend on core crates only
3. **Core** → may depend on other core crates only
4. Core/Modules must NEVER depend on apps

## Common Patterns

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
    // Return Ok(...) or Err(NestError::...)
}
```

### Loading configuration

```rust
use nest_config::Config;

let config = Config::load("config.toml")?;
```

### Setting up logging

```rust
use nest_logging::init;

init()?;
tracing::info!("Application started");
```

### Building a Tauri desktop app

```rust
// In src-tauri/src/main.rs or lib.rs
use nest_tauri::TauriHost;

let host = TauriHost::builder()
    .with_module(my_module)
    .build()?;
```

## Workspace Setup

Apps typically reference Nest crates via git dependency:

```toml
# In app's Cargo.toml
[dependencies]
nest-core = { git = "https://github.com/pacificnm/nest", branch = "main" }
nest-error = { git = "https://github.com/pacificnm/nest", branch = "main" }
```

Or via path when checked out locally:

```toml
# In app's .cargo/config.toml
[patch."https://github.com/pacificnm/nest"]
nest-core = { path = "../../../core/crates/nest-core" }
nest-error = { path = "../../../core/crates/nest-error" }
```

## Questions to Answer

1. Which Nest crate should I use for [X]?
2. How do I add [nest-*] crate to my app?
3. What's the dependency chain for [nest-*]?
4. Show me examples of using [nest-*] crate
5. How do Nest crates follow the architecture layers?

## Related Files

- `/data/projects/nest/docs/architecture.md` - Layer architecture
- `/data/projects/nest/core/crates/` - All core crate sources
- `/data/projects/nest/modules/crates/` - All module sources
- `/data/projects/nest/apps/README.md` - App development guide
