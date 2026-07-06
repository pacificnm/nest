# nest-config v1 Implementation Plan

## Status: Implemented

See [nest-config docs](../nest-config/README.md).

## Context

Configuration loading extracted from `nest-cli::config` into a standalone foundation crate usable by CLI, desktop, server, and tests.

**Core rule:** `nest-config` loads and exposes configuration. It does not interpret module-specific settings.

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-config` | `ConfigLoader`, `ConfigDocument`, `ConfigService`, paths, format detection |
| `nest-error` | `NEST_CONFIG_*` error codes |
| Hosts (`nest-cli`, `nest-tui`, `nest-tauri`) | Call `ConfigLoader`, register `ConfigService` via `nest-core` |

`nest-config` has **no** `nest-core` dependency. `ConfigService` is a plain `Clone` type; hosts call `app.register_service(ConfigService::new(loaded))`.

## Public API

- `ConfigDocument` — TOML-first parsing; optional JSON behind `json` feature
- `ConfigService` — section access with source/path metadata
- `ConfigSource` — `File`, `SearchDefaults`, `Memory`
- `ConfigFormat` — `Toml`, `Json` (feature), `Auto`
- `ConfigLoader` / `LoadedConfig` — load pipeline
- `default_search_paths` / `resolve_search` — default file discovery

## Default search paths

1. `./config.toml`
2. `./config/config.toml`
3. `~/.config/{app_name}/config.toml`

When none exist, `SearchDefaults` yields an empty in-memory document.

## Error codes

| Code | When |
|------|------|
| `NEST_CONFIG_NOT_FOUND` | Explicit file missing |
| `NEST_CONFIG_READ_FAILED` | File exists but read fails |
| `NEST_CONFIG_PARSE_FAILED` | Syntax / root type invalid |
| `NEST_CONFIG_SECTION_MISSING` | `section()` on absent section |
| `NEST_CONFIG_SECTION_INVALID` | Section deserialize failure |
| `NEST_CONFIG_UNSUPPORTED_FORMAT` | Unknown extension or format mismatch |

## Module consumption pattern

Each module defines its own `#[derive(Deserialize)]` section struct and reads in `configure`:

```rust
let config = app.service::<ConfigService>()?;
let file_config = config
    .optional_section::<FileConfig>("file")?
    .unwrap_or_default();
```

v1 does **not** wire `FileModule` / `CsvModule` to auto-read config — that is per-module follow-up work.

## Deferred

- Live reload
- Remote config / secrets / env substitution
- Profile merging (`dev.toml` + `local.toml`)
- Schema validation / GUI editor
- `nest-config` Module trait (hosts register service manually)
