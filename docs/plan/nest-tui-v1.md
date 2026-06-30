# nest-tui v1 Implementation Plan

## Status: Implemented

See [nest-tui docs](../nest-tui/README.md).

## Context

Terminal UI host for Nest applications using Ratatui and crossterm. Mirrors `nest-cli` bootstrap patterns with a long-running event loop and terminal lifecycle management.

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-tui` | `TuiApp`, `TuiScreen`, terminal setup/restore, event loop |
| `nest-config` | `ConfigLoader`, `ConfigService` |
| `nest-logging` | File-only default via `LoggingConfig::for_tui` |
| `nest-core` | `AppBuilder`, modules, services |

## Bootstrap order

1. Parse startup options
2. Load config
3. Initialize logging (file only — never stdout during TUI)
4. Build `AppContext`
5. Register modules/services
6. Initialize terminal
7. Run Ratatui event loop
8. Restore terminal on exit/panic

**Precedence:** defaults < config file < CLI flags

## Deferred

- `nest-cli-theme` adapter crate
- Command palette, dockable panels
- `MemorySink` log panel (nest-logging v2)
- Module auto-config from `[file]` etc.
