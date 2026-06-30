# API reference

**Crate:** `nest_logging`  
**Version:** 0.1.0

## Prelude

```rust
use nest_logging::prelude::*;
```

## Entry points

| Function | Description |
|----------|-------------|
| `init(config)` | Install subscriber; store guard globally |
| `init_logging(config)` | Install subscriber; return `LoggingGuard` |
| `log_error(error)` | Structured error event |
| `log_result(result)` | Log if `Err` |
| `cleanup_logs(dir, app_name, policy)` | Retention sweeper |

## LoggingConfig builder

| Method | Description |
|--------|-------------|
| `new(app_name)` | Create config |
| `with_console()` | Add console target |
| `with_file(directory)` | Add text file target |
| `with_json_file(directory)` | Add JSON file target |
| `with_default_level(level)` | Default EnvFilter level |
| `with_module_level(target, level)` | Per-target override |
| `with_format(format)` | Pretty / Compact / Json |
| `with_rotation(policy)` | Never / Daily / Hourly |
| `with_retention(policy)` | Disabled / Days / Files |
| `capture_panics(bool)` | Install panic hook |
| `env_override(bool)` | Honor `RUST_LOG` (default true) |

## Enums

- `LogLevel`: Trace, Debug, Info, Warn, Error
- `LogFormat`: Pretty, Compact, Json
- `LogTarget`: Console, File, JsonFile
- `RetentionPolicy`: Disabled, Days(u32), Files(u32)
- `RotationPolicy`: Never, Daily, Hourly, SizeBytes(u64) (unsupported v1)

## Error codes

Defined in `nest_logging::codes`:

- `NEST_LOGGING_FILTER_INVALID`
- `NEST_LOGGING_DIR_CREATE`
- `NEST_LOGGING_ALREADY_INIT`
- `NEST_LOGGING_ROTATION_UNSUPPORTED`
- `NEST_LOGGING_NO_TARGETS`
- `NEST_LOGGING_NO_DIRECTORY`

## Dependencies

| Crate | Required by nest-logging |
|-------|------------------------|
| `tracing` | Yes |
| `tracing-subscriber` | Yes |
| `tracing-appender` | Yes |
| `nest-error` | Yes (for `log_error`, `NestResult`) |
| `nest-core` | No |
