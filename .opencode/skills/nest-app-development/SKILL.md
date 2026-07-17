---
name: nest-app-development
description: Use when developing an app under apps/ that uses the Nest framework — integrating Nest crates, choosing between crates, setting up app architecture, dependencies, or project structure.
---

# Nest App Development Skill

**Purpose**: Guide developers in building applications using Nest shared crates from `core/crates/` and `modules/crates/`.

## When to use

- User is developing an app in `apps/`
- User needs to integrate Nest framework features
- User is choosing between Nest crates for app functionality
- User is setting up app architecture, dependencies, or project structure

Every code example below is checked against current crate source, not
paraphrased from memory — verify against the real crate if the API may have
moved since.

## App Types and Host Choices

### Desktop App (Tauri + React + Tailwind)

**Use**: `nest-tauri` host

**Structure**:
```
apps/my-app/
  ui/                 # React + TypeScript + Tailwind
  src-tauri/          # Rust: Tauri commands, Nest modules
    Cargo.toml
    src/
      main.rs
      commands.rs
```

**Dependencies** (`src-tauri/Cargo.toml`):
```toml
[dependencies]
nest-tauri = { workspace = true }
nest-core = { workspace = true }
nest-config = { workspace = true }
nest-logging = { workspace = true }
nest-error = { workspace = true }
nest-design = { workspace = true }
nest-theme = { workspace = true }
nest-react-theme = { workspace = true }
```

**Frontend** (`ui/`):
- React + TypeScript
- Tailwind CSS
- Vite build tool
- Uses CSS variables from `nest-design` / `nest-theme`

### CLI App

**Use**: `nest-cli` host

**Structure**:
```
apps/my-cli/
  crates/
    core/           # App-specific logic
    cli/            # CLI entry point
  Cargo.toml
```

**Dependencies**:
```toml
[dependencies]
nest-cli = { workspace = true }
nest-core = { workspace = true }
nest-logging = { workspace = true }
nest-error = { workspace = true }
```

### TUI App (Terminal UI)

**Use**: `nest-tui` host

**Dependencies**:
```toml
[dependencies]
nest-tui = { workspace = true }
nest-core = { workspace = true }
nest-logging = { workspace = true }
```

## Common App Patterns

### Pattern 1: Basic Tauri App Setup

**File**: `src-tauri/src/main.rs`

```rust
use nest_tauri::TauriApp;

fn main() {
    TauriApp::new("my-app").run(tauri::generate_context!());
}
```

The bootstrap type is `TauriApp` — `TauriApp::new(app_name).module(m).run(context)`.
`run` takes a `tauri::Context` (typically from `tauri::generate_context!()`)
and never returns; use `try_run` instead if you need a `Result` you can
handle rather than a process exit.

### Pattern 2: Adding a Module to App

**File**: `src-tauri/src/main.rs`

```rust
use nest_tauri::TauriApp;
use nest_airtable::AirtableModule; // example module

fn main() {
    TauriApp::new("my-app")
        .module(AirtableModule::new(/* config */))
        .run(tauri::generate_context!());
}
```

### Pattern 3: Tauri Commands

**File**: `src-tauri/src/commands.rs`

IPC handlers should be thin and bridge `NestError` to a string for the
webview — don't return `NestResult` directly from a `#[tauri::command]`
(`NestError` isn't `Serialize`):

```rust
use nest_core::AppContext;
use nest_error::NestResult;
use nest_tauri::NestHostState;
use tauri::State;

async fn get_data(ctx: &AppContext) -> NestResult<Vec<Data>> {
    let svc = ctx.service::<DataService>()?;
    svc.find_all().await
}

#[tauri::command]
pub async fn get_data_cmd(state: State<'_, NestHostState>) -> Result<Vec<Data>, String> {
    get_data(&state.context).await.map_err(|e| e.to_string())
}
```

`NestHostState.context` is a public `Arc<AppContext>` **field**, not a
`.context()` method.

### Pattern 4: Configuration Loading

**File**: `src-tauri/src/main.rs`

```rust
use nest_config::{ConfigLoader, ConfigService};
use nest_logging::prelude::*;

#[derive(serde::Deserialize)]
struct AppConfig {
    app_name: String,
    debug: bool,
}

fn main() -> nest_error::NestResult<()> {
    let loaded = ConfigLoader::new("my-app").load()?;
    let config_service = ConfigService::new(loaded);
    let config: AppConfig = config_service.section("app")?;

    init(LoggingConfig::new("my-app").with_console())?;

    tracing::info!("Starting {}", config.app_name);
    Ok(())
}
```

There is no `Config::load(path)` convenience constructor, and
`nest_logging::init` takes a required `LoggingConfig` argument — there is no
zero-argument `init()`.

### Pattern 5: Error Handling

**File**: `src-tauri/src/service.rs`

```rust
use nest_error::{NestError, NestResult};

pub fn process_data(input: &str) -> NestResult<ProcessedData> {
    if input.is_empty() {
        return Err(NestError::validation("Input cannot be empty"));
    }

    Ok(ProcessedData::new(input))
}

#[tauri::command]
pub fn handle_command(input: String) -> Result<ProcessedData, String> {
    process_data(&input).map_err(|e| e.to_string())
}
```

The constructor is `NestError::validation(...)`, not `validation_error(...)`.

### Pattern 6: HTTP Client Usage

**File**: `src-tauri/src/api.rs`

```rust
use nest_http_client::HttpClientService;
use nest_error::NestResult;

pub struct MyService {
    http: HttpClientService,
}

impl MyService {
    pub async fn fetch_data(&self, url: &str) -> NestResult<MyResponseType> {
        self.http.get_json(url).await
    }
}
```

The client type is `HttpClientService` (there is no bare `HttpClient` type),
and the fetch method is `get_json::<T>(url) -> NestResult<T>` — it
deserializes directly into the type you ask for, it doesn't return a raw
`Response`.

### Pattern 7: Background Tasks

**File**: `src-tauri/src/tasks.rs`

```rust
use async_trait::async_trait;
use nest_error::NestResult;
use nest_task::{Task, TaskContext};
use nest_task_runtime::TaskManagerService; // implements the TaskManager trait

pub struct MyTask;

#[async_trait]
impl Task for MyTask {
    type Output = ();

    fn name(&self) -> &'static str {
        "my-task"
    }

    async fn run(&self, ctx: TaskContext) -> NestResult<()> {
        // Task logic. ctx.cancel_token().is_cancelled() to check for
        // cooperative cancellation — it's a poll, not an awaitable future.
        Ok(())
    }
}

// Elsewhere, given `manager: &impl nest_task::TaskManager`:
// manager.spawn(MyTask).await?;
```

`Task::run` takes `&self`, not `&mut self` — a task needing mutable state
across `run` calls (e.g. a scheduler with an internal interval loop) needs
interior mutability (`tokio::sync::Mutex`). `TaskManager::spawn` runs a task
**once**; there's no built-in interval/recurring scheduling — a
long-running task manages its own loop inside `run`. `TaskRuntime` (from the
same crate) is a different type — a raw Tokio runtime handle wrapper with no
`spawn` method of its own; don't confuse it with `TaskManager`.

### Pattern 8: Caching

**File**: `src-tauri/src/cache.rs`

```rust
use nest_cache::{Cache, CacheKey};
use nest_error::NestResult;

pub struct DataService {
    cache: Cache,
}

impl DataService {
    pub async fn get_data(&self, key: &CacheKey) -> NestResult<Data> {
        if let Some(cached) = self.cache.get_json::<Data>(key)? {
            return Ok(cached);
        }

        let data = self.fetch_from_source().await?;
        self.cache.set_json(key, &data)?;
        Ok(data)
    }
}
```

`Cache` is keyed by `CacheKey`, not a bare `&str`, and the methods are
`get_json` / `set_json` (there's also `get_bytes` / `set_bytes` for raw
data) — both are synchronous and return `CacheResult<...>`, not a bare
`Option<T>`.

## App Development Workflow

### 1. Choose host

- Desktop GUI → `nest-tauri`
- CLI → `nest-cli`
- Terminal UI → `nest-tui`

### 2. Set up dependencies

Add Nest crates to `Cargo.toml` (see `add-nest-dependency` tool)

### 3. Initialize host

```rust
TauriApp::new("my-app")
    .module(MyModule)
    .run(tauri::generate_context!());
```

### 4. Add modules (optional)

Use existing modules from `modules/crates/` or create custom ones

### 5. Implement features

Use Nest crates for:
- Error handling → `nest-error`
- Config → `nest-config`
- Logging → `nest-logging`
- Validation → `nest-validation`
- HTTP → `nest-http-client`, `nest-http-serve`
- Data → `nest-data`
- Cache → `nest-cache`
- Tasks → `nest-task`, `nest-task-runtime`

### 6. Build UI (Tauri apps)

React + TypeScript + Tailwind in `ui/`

## Module Registration

Modules implement `nest_core::Module` and are registered via `AppBuilder`
(`AppBuilder` lives in `nest-core`, not `nest-app`):

```rust
use nest_core::{AppBuilder, Module, ModuleId};
use nest_error::NestResult;

pub struct MyModule;

impl Module for MyModule {
    fn id(&self) -> ModuleId {
        ModuleId("my-module")
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        app.register_service(MyService::new())
    }
}
```

`Module::configure` (not `register`) takes `&mut AppBuilder` and returns
`NestResult<()>`. `id()` returning a stable `ModuleId` is required —
duplicate ids are rejected at build time. `AppBuilder::register_service`
takes the service **instance**, not just a type parameter, and returns a
`NestResult<()>` you should propagate with `?`. There is no
`register_repository` method — repositories are typically registered as
regular services via `register_service`.

## Testing Apps

```bash
# Every Nest product exposes the same ./build verbs — see docs/build.md
./build test    # Run tests
./build check   # CI checks (fmt, clippy, tests)
./build dev     # Development (hot reload / cargo run / tauri dev)
./build run     # Build if needed, then launch
```

Plain `cargo test` / `cargo check` also work directly against a crate, but
prefer `./build` for whole-product workflows — it wraps the right profile
(`rust`, `tauri`, `node`, `workspace`) per product.

## Related Tools

- `find-nest-crate` - Find the right Nest crate for a need
- `check-nest-dependencies` - Verify dependency rules
- `add-nest-dependency` - Add Nest crate to app

## Reference Apps

- `apps/loon/` - Real product, actively developed ([pacificnm/loon](https://github.com/pacificnm/loon)) — HTTP server + desktop admin + webOS client
- `apps/swift/` - Local checkout, personal PM + knowledge + Ollama assistant
- `apps/airtable-sync/` - External repo (Airtable integration)
- `kiwi`, `finch` - Planned, not yet started

See [apps/README.md](../../apps/README.md) for current status.
