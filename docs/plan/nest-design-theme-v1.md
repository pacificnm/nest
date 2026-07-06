# nest-design + nest-theme v1 Implementation Plan

## Status: Implemented

See [nest-design docs](../nest-design/README.md) and [nest-theme docs](../nest-theme/README.md).

## Context

Shared theme language before platform adapters. `nest-design` owns the token schema; `nest-theme` owns lifecycle; `nest-react-theme` (desktop) and `nest-cli-theme` (terminal) are platform adapters.

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-design` | Token structs, `ThemeDefinition`, built-in light/dark, serde |
| `nest-theme` | `ThemeService`, `ThemeModule`, loader, validator, registry, `ThemeAdapter` trait |
| `nest-core` | Unchanged — theme-agnostic module + service registry |

## nest-design

- `ColorTokens`, `SpacingTokens`, `RadiusTokens`, `TypographyTokens`, `StatusTokens`
- `ThemeId`, `ThemeMode`, `ThemeDefinition`
- `themes::light()`, `themes::dark()`
- Colors as `#RRGGBB` / `#RRGGBBAA` hex strings

## nest-theme

- `ThemeRegistry` — in-memory map by id
- `ThemeValidator` — required tokens + color format
- `ThemeLoader` — TOML + JSON from file or string
- `ThemeService` — `RwLock` for registry + active id (shared `&T` from `AppContext`)
- `ThemeModule` — registers `ThemeService` with optional defaults
- `ThemeAdapter<TOutput>` — contract for future platform crates

## v1 limitations

- No theme change events (`nest-events` later)
- No user preference persistence (`nest-settings` later)
- No `ShadowTokens`, `ComponentTokens`, `ThemeVariant`
- No platform adapter crates

## Follow-up

- `nest-react-theme` — CSS vars + Tailwind preset for Tauri/React apps (see [plan](./nest-tauri-v1.md))
- `nest-cli-theme` — Ratatui styling
- `ThemeChanged` event
- Settings integration for persisted theme preference
