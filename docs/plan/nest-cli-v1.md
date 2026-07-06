# nest-cli v1 Implementation Plan

## Status: Implemented

See [nest-cli docs](../nest-cli/README.md).

## Context

Command-line host for Nest applications — the non-interactive host alongside `nest-tui` (terminal) and `nest-tauri` (desktop). Wires modules, parses commands, initializes logging and configuration before the module graph, renders errors, and exits cleanly.

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-cli` | `CliApp`, `CliCommand`, bootstrap, error render |
| `nest-config` | `ConfigLoader`, `ConfigDocument`, `ConfigService` |
| `nest-core` | `AppBuilder`, modules, services |
| `nest-logging` | Host initializes via `CliApp` (no `LoggingModule`) |
| `nest-error` | `NestErrorReport`, exit code mapping |

## Bootstrap order

1. Parse CLI globals + subcommand
2. Load config TOML (`--config` or default search)
3. Initialize logging
4. Build `AppContext` + register modules
5. Run command
6. Render errors / exit codes

## v1 configuration

Configuration lives in the standalone [`nest-config`](../nest-config/README.md) crate. `nest-cli` loads via `ConfigLoader::file_or_search` and registers `ConfigService` during bootstrap.

## Deferred

- `nest-cli-theme`, completions, miette rendering (TUI host: [nest-tui](../nest-tui/README.md))
- Example `airtable-sync` binary (separate project crate)
