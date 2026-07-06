# nest-theme Documentation

`nest-theme` owns theme lifecycle: loading, validation, registration, and active theme state. It registers [`ThemeService`](../../core/crates/nest-theme/src/service.rs) via [`ThemeModule`](../../core/crates/nest-theme/src/module.rs) into `nest-core`'s service registry. `nest-core` stays theme-agnostic.

**Crate path:** [`core/crates/nest-theme`](../../core/crates/nest-theme)

## Quick start

```rust
use nest_core::AppBuilder;
use nest_design::ThemeId;
use nest_theme::{ThemeModule, ThemeService};

let built = AppBuilder::new()
    .module(ThemeModule::default())
    .build()?;

let themes = built.context.service::<ThemeService>()?;
themes.set_active_theme(&ThemeId::from("nest-dark"))?;
let active = themes.active_theme()?;
```

## Loading custom themes

```rust
themes.load_and_register("themes/custom.toml")?;
```

Supported formats: `.toml`, `.json`. Files are validated before registration.

## ThemeService API

| Method | Description |
|--------|-------------|
| `register_theme` | Register a `ThemeDefinition` |
| `load_and_register` | Load from disk, validate, register |
| `set_active_theme` | Set active theme by id |
| `active_theme` / `active_id` | Read current theme |
| `theme` | Clone a registered theme by id |
| `list_themes` | All registered ids |

`ThemeModule::default()` registers built-in light/dark themes and sets `nest-light` active.

## ThemeAdapter trait

Platform crates implement conversion from tokens to host output:

```rust
pub trait ThemeAdapter<TOutput> {
    fn adapt(theme: &ThemeDefinition) -> NestResult<TOutput>;
}
```

Deferred v2+ crates: `nest-react-theme` (desktop — **implemented**), `nest-cli-theme` (terminal).

## Error codes

| Code | Meaning |
|------|---------|
| `NEST_THEME_NOT_FOUND` | Unknown theme id |
| `NEST_THEME_ALREADY_REGISTERED` | Duplicate registration |
| `NEST_THEME_NO_ACTIVE` | No active theme set |
| `NEST_THEME_FORMAT_UNSUPPORTED` | Bad file extension |

## Related

- [Implementation plan](../plan/nest-design-theme-v1.md)
- [nest-design](../nest-design/README.md) — token schema
