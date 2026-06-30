# Configuration

## LoggingConfig

```rust
pub struct LoggingConfig {
    pub app_name: String,
    pub level: LogLevel,
    pub module_levels: HashMap<String, LogLevel>,
    pub targets: Vec<LogTarget>,
    pub format: LogFormat,
    pub directory: Option<PathBuf>,
    pub retention: RetentionPolicy,
    pub rotation: RotationPolicy,
    pub capture_panics: bool,
    pub env_override: bool,  // default: true
}
```

## Builder example

```rust
LoggingConfig::new("kiwi")
    .with_console()
    .with_file("./logs")
    .with_json_file("./logs")
    .with_default_level(LogLevel::Info)
    .with_module_level("nest_data", LogLevel::Debug)
    .with_format(LogFormat::Pretty)
    .with_rotation(RotationPolicy::Daily)
    .with_retention(RetentionPolicy::Days(14))
    .capture_panics(true)
```

## Log targets

| Target | Description |
|--------|-------------|
| `Console` | stdout via fmt layer |
| `File` | Rolling text log (`{app_name}.log`) |
| `JsonFile` | Rolling JSON log (`{app_name}-json.log`) |

File targets require `.with_file()` / `.with_json_file()` which sets `directory`.

## Log formats

| Format | Use |
|--------|-----|
| `Pretty` | Developer console (ANSI, multi-line) |
| `Compact` | Single-line text |
| `Json` | NDJSON for production |

Console and text file layers use `format`. `JsonFile` target always writes JSON.

## Rotation

| Policy | v1 support |
|--------|------------|
| `Never` | Yes |
| `Daily` | Yes (`tracing-appender`) |
| `Hourly` | Yes |
| `SizeBytes(n)` | No — returns `NEST_LOGGING_ROTATION_UNSUPPORTED` |

## Retention

| Policy | Behavior |
|--------|----------|
| `Disabled` | No cleanup |
| `Days(n)` | Delete log files older than N days at init |
| `Files(n)` | Keep newest N matching files at init |

Call `cleanup_logs(directory, app_name, policy)` manually for periodic sweeps.

## Initialization

```rust
// Guard stored internally for process lifetime
init(config)?;

// Caller holds WorkerGuard(s) for file writers
let guard = init_logging(config)?;
```

`init()` fails if a global subscriber is already set (`NEST_LOGGING_ALREADY_INIT`).
