# nest-gui v1 Implementation Plan

## Status: Implemented

See [nest-gui docs](../nest-gui/README.md).

## Context

Desktop GUI host for Nest applications using eframe and egui. Mirrors `nest-cli` / `nest-tui` bootstrap patterns with an eframe main loop.

**Naming:** [`nest-gui`](../nest-gui/README.md) is the **desktop host**. [`nest-app`](../nest-app/README.md) is the **shared application container** between `nest-core` and all hosts — not a GUI crate.

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-app` | `NestApp`, metadata, lifecycle orchestration (optional pre-build in `main`) |
| `nest-gui` | `GuiApp`, `GuiView`, window shell, eframe loop |
| `nest-theme` | `ThemeModule`, `ThemeService` (user registers explicitly) |
| `nest-config` | `ConfigLoader`, `ConfigService` |
| `nest-logging` | File-only default via `LoggingConfig::for_gui` |

## Bootstrap order

1. Parse startup options
2. Load config
3. Initialize logging (file only)
4. Build `AppContext`
5. Register modules/services
6. Start eframe main loop
7. Shutdown on exit

**Precedence:** defaults < config file < CLI flags

## Deferred

- `nest-egui-theme` full token adapter
- In-app log viewer (`MemorySink` in nest-logging v2)
- `nest-ui` components, docking, command palette
- `nest-http-server` host
