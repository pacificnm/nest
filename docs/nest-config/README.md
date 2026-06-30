# nest-config

Configuration loading for the [Nest framework](../../README.md).

**Crate path:** [`crates/nest-config`](../../crates/nest-config)

`nest-config` loads, parses, and exposes application configuration. It does not depend on `nest-core`; hosts register [`ConfigService`](../../crates/nest-config/src/service.rs) into the Nest service registry after loading.

## Quick start

```rust
use nest_config::{ConfigLoader, ConfigService, ConfigSource};
use nest_core::AppBuilder;

let loaded = ConfigLoader::file_or_search("my-app", None).load()?;
let document = loaded.document.clone();

let mut builder = AppBuilder::new("my-app");
builder.register_service(ConfigService::new(loaded))?;
```

## Configuration sources

| Source | Behavior |
|--------|----------|
| `ConfigSource::File(path)` | File must exist; read and parse |
| `ConfigSource::SearchDefaults` | First hit from default paths, or empty document |
| `ConfigSource::Memory(doc)` | Pre-built document (tests, embedded defaults) |

### Default search paths

When `--config` is omitted (CLI) or `SearchDefaults` is used:

1. `./config.toml`
2. `./config/config.toml`
3. `~/.config/{app_name}/config.toml`

```rust
use nest_config::default_search_paths;

for path in default_search_paths("my-app") {
    println!("{}", path.display());
}
```

## Section access

Modules own their section types. `nest-config` only deserializes into types you provide.

```toml
[logging]
level = "info"

[data.sqlite]
database = "./data/app.db"
```

```rust
use nest_config::ConfigService;
use nest_core::AppContext;
use serde::Deserialize;

#[derive(Deserialize)]
struct LoggingSection {
    level: String,
}

fn run(ctx: &AppContext) -> nest_error::NestResult<()> {
    let config = ctx.service::<ConfigService>()?;

    let logging: LoggingSection = config.section("logging")?;
    let sqlite = config.optional_section::<SqliteSection>("data.sqlite")?;

    Ok(())
}
```

- `section()` → `NEST_CONFIG_SECTION_MISSING` when absent; `NEST_CONFIG_SECTION_INVALID` on deserialize failure
- `optional_section()` → `Ok(None)` when absent; errors only on invalid data
- Dotted section names (e.g. `data.sqlite`) traverse nested TOML tables

## Formats

| Format | Notes |
|--------|-------|
| `ConfigFormat::Toml` | Default |
| `ConfigFormat::Auto` | Infer from `.toml` / `.json` extension |
| `ConfigFormat::Json` | Requires `json` feature |

```toml
nest-config = { path = "../nest-config", features = ["json"] }
```

## Error codes

| Code | When |
|------|------|
| `NEST_CONFIG_NOT_FOUND` | Explicit file missing |
| `NEST_CONFIG_READ_FAILED` | File exists but read fails |
| `NEST_CONFIG_PARSE_FAILED` | Parse failure |
| `NEST_CONFIG_SECTION_MISSING` | Required section absent |
| `NEST_CONFIG_SECTION_INVALID` | Section present but invalid |
| `NEST_CONFIG_UNSUPPORTED_FORMAT` | Unknown extension or format mismatch |

## CLI integration

`nest-cli` uses `ConfigLoader::file_or_search` during bootstrap and registers `ConfigService` before modules run. See [nest-cli configuration](../nest-cli/README.md#configuration).

## Related

- [Implementation plan](../plan/nest-config-v1.md)
- [nest-cli](../nest-cli/README.md) — CLI host
- [nest-error](../nest-error/README.md) — error codes
