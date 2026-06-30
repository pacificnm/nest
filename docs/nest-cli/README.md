# nest-cli

Command-line host for the [Nest framework](../../README.md).

**Crate path:** [`core/crates/nest-cli`](../../core/crates/nest-cli)

## Quick start

```rust
use nest_cli::{CliApp, CliCommand};
use nest_core::AppContext;
use nest_error::NestResult;
use nest_file::{FileModule, FileServiceConfig};
use nest_file_csv::{CsvModule, CsvOptions, CsvService};
use nest_logging::LoggingConfig;

fn main() {
    CliApp::new("airtable-sync")
        .with_logging(LoggingConfig::for_cli("airtable-sync"))
        .with_log_level_from_args(true)
        .module(FileModule::with_config(
            FileServiceConfig::scoped("./workspace").allow_create_dirs(true),
        ))
        .module(CsvModule)
        .command(ValidateCsvCommand)
        .run();
}
```

## Bootstrap order

| Step | What happens |
|------|----------------|
| 1 | Parse global flags and subcommand |
| 2 | Load config (`--config` or default paths) |
| 3 | `nest_logging::init` from defaults + `[logging]` + flags |
| 4 | `AppBuilder` — register `ConfigService`, modules |
| 5 | Run `CliCommand` |
| 6 | On error: `NestErrorReport` → stderr, exit code |

**Rule:** Hosts initialize logging. Modules only emit `tracing` events.

## Global flags

| Flag | Effect |
|------|--------|
| `--config <path>` | Load explicit config file |
| `--log-level <level>` | Override log level |
| `--log-file <path>` | Enable file logging |
| `--json` | JSON error output |
| `--quiet` | Suppress non-error stdout |
| `--verbose` | Debug logging |
| `--no-color` | Plain error output |

## Configuration

Default search when `--config` is omitted:

1. `./config.toml`
2. `./config/config.toml`
3. `~/.config/{app_name}/config.toml`

If none exist, an empty document is used.

```toml
[logging]
level = "info"
directory = "./logs"

[data.sqlite]
database = "./data/app.db"
```

```rust
use nest_config::ConfigService;

let config = ctx.service::<ConfigService>()?;
let section = config.section::<SqliteSection>("data.sqlite")?;
```

See [nest-config](../nest-config/README.md) for loaders, sources, and error codes.

## Commands

```rust
impl CliCommand for MyCommand {
    fn name(&self) -> &'static str { "sync" }
    fn about(&self) -> &'static str { "Sync data" }
    fn configure(&self, cmd: clap::Command) -> clap::Command { cmd }
    fn run(&self, ctx: &AppContext, matches: &clap::ArgMatches) -> NestResult<()> {
        Ok(())
    }
}
```

## Async commands

Enable the `async` feature and register `AsyncCliCommand`. When async commands are present, `nest-cli` auto-inserts `TaskRuntimeModule` if not already registered.

```toml
nest-cli = { path = "../nest-cli", features = ["async"] }
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Validation |
| 3 | Config |
| 4 | File/I/O |
| 5 | Network |
| 6 | Data |
| 10 | Usage |

## Related

- [nest-core](../nest-core/application.md) — modules and services
- [nest-logging](../nest-logging/README.md) — logging init (host-owned)
- [nest-config](../nest-config/README.md) — configuration loading
- [Implementation plan](../plan/nest-cli-v1.md)
