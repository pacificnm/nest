# Nest App Development Skill

**Purpose**: Guide developers in building applications using Nest shared crates from `core/crates/` and `modules/crates/`.

## When to use

- User is developing an app in `apps/`
- User needs to integrate Nest framework features
- User is choosing between Nest crates for app functionality
- User is setting up app architecture, dependencies, or project structure

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
nest-tauri = { workspace = true, features = ["runtime"] }
nest-core = { workspace = true }
nest-app = { workspace = true }
nest-config = { workspace = true }
nest-logging = { workspace = true }
nest-error = { workspace = true, features = ["serde"] }
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
use nest_tauri::TauriHost;
use nest_logging::init as init_logging;

fn main() {
    init_logging().expect("Failed to initialize logging");
    
    let host = TauriHost::builder()
        .build()
        .expect("Failed to build host");
    
    host.run();
}
```

### Pattern 2: Adding a Module to App

**File**: `src-tauri/src/main.rs`

```rust
use nest_tauri::TauriHost;
use nest_airtable::AirtableModule;  // Example module

fn main() {
    let host = TauriHost::builder()
        .with_module(AirtableModule::new())
        .build()
        .expect("Failed to build host");
    
    host.run();
}
```

### Pattern 3: Tauri Commands

**File**: `src-tauri/src/commands.rs`

```rust
use nest_error::{NestError, NestResult};
use tauri::State;

#[tauri::command]
pub fn get_data(state: State<MyState>) -> NestResult<Vec<Data>> {
    state.repository.find_all()
        .map_err(|e| NestError::from(e))
}

#[tauri::command]
pub fn create_item(data: CreateItemDto) -> NestResult<Item> {
    // Validate
    nest_validation::validate(&data)?;
    
    // Process
    Ok(Item::from(data))
}
```

### Pattern 4: Configuration Loading

**File**: `src-tauri/src/main.rs`

```rust
use nest_config::Config;
use nest_logging::init;

#[derive(serde::Deserialize)]
struct AppConfig {
    app_name: String,
    debug: bool,
}

fn main() {
    let config: Config<AppConfig> = Config::load("config.toml")
        .expect("Failed to load config");
    
    init().expect("Failed to initialize logging");
    
    tracing::info!("Starting {}", config.app_name);
}
```

### Pattern 5: Error Handling

**File**: `src-tauri/src/service.rs`

```rust
use nest_error::{NestError, NestResult};

pub fn process_data(input: &str) -> NestResult<ProcessedData> {
    if input.is_empty() {
        return Err(NestError::validation_error("Input cannot be empty"));
    }
    
    // Process
    Ok(ProcessedData::new(input))
}

#[tauri::command]
pub fn handle_command(input: String) -> Result<ProcessedData, String> {
    process_data(&input).map_err(|e| e.to_string())
}
```

### Pattern 6: HTTP Client Usage

**File**: `src-tauri/src/api.rs`

```rust
use nest_http_client::HttpClient;
use nest_error::NestResult;

pub struct MyService {
    http: HttpClient,
}

impl MyService {
    pub async fn fetch_data(&self, url: &str) -> NestResult<Response> {
        self.http.get(url).await
    }
}
```

### Pattern 7: Background Tasks

**File**: `src-tauri/src/tasks.rs`

```rust
use nest_task::Task;
use nest_task_runtime::TaskRuntime;

pub struct MyTask {
    // task data
}

#[async_trait]
impl Task for MyTask {
    async fn execute(&self) -> NestResult<()> {
        // Task logic
        Ok(())
    }
}

// Register task
runtime.spawn(MyTask { /* ... */ });
```

### Pattern 8: Caching

**File**: `src-tauri/src/cache.rs`

```rust
use nest_cache::Cache;

pub struct DataService {
    cache: Cache<Data>,
}

impl DataService {
    pub async fn get_data(&self, key: &str) -> NestResult<Data> {
        if let Some(cached) = self.cache.get(key) {
            return Ok(cached);
        }
        
        let data = self.fetch_from_source().await?;
        self.cache.set(key, data.clone());
        Ok(data)
    }
}
```

## App Development Workflow

### 1. Choose host

- Desktop GUI → `nest-tauri`
- CLI → `nest-cli`
- Terminal UI → `nest-tui`

### 2. Set up dependencies

Add Nest crates to `Cargo.toml` (see `add-nest-dependency` tool)

### 3. Initialize host

```rust
let host = TauriHost::builder()
    .with_module(MyModule)
    .build()?;
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

Modules extend app functionality:

```rust
use nest_core::Module;
use nest_app::AppBuilder;

pub struct MyModule;

impl Module for MyModule {
    fn register(&self, builder: &mut AppBuilder) {
        builder.register_service::<MyService>();
        builder.register_repository::<MyRepository>();
    }
}
```

## Testing Apps

```bash
# Run tests
cargo test

# Check compilation
cargo check

# Run app (Tauri)
cd src-tauri && cargo tauri dev

# Run app (CLI)
cargo run
```

## Related Tools

- `find-nest-crate` - Find the right Nest crate for a need
- `check-nest-dependencies` - Verify dependency rules
- `add-nest-dependency` - Add Nest crate to app

## Reference Apps

- `apps/swift/` - Reference Tauri desktop app (PM + knowledge + AI)
- `apps/kiwi/` - Planned app
- `apps/loon/` - Planned app
- `apps/airtable-sync/` - External repo (Airtable integration)
