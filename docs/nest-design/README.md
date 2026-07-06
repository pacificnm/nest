# nest-design Documentation

`nest-design` owns the shared design token schema and built-in theme definitions for Nest. It has no runtime logic and no dependency on `nest-core` or UI libraries.

**Crate path:** [`core/crates/nest-design`](../../core/crates/nest-design)

## Quick start

```rust
use nest_design::{ThemeDefinition, ThemeMode, themes};

let light: ThemeDefinition = themes::light();
assert_eq!(light.id.as_str(), "nest-light");
assert_eq!(light.mode, ThemeMode::Light);
```

## Token groups

| Group | Type | Description |
|-------|------|-------------|
| Colors | `ColorTokens` | Semantic UI colors (`background`, `primary`, …) |
| Spacing | `SpacingTokens` | Scale in logical pixels (`xs` … `xl`) |
| Radius | `RadiusTokens` | Border radius scale |
| Typography | `TypographyTokens` | Named text roles (`body`, `heading`, …) |
| Status | `StatusTokens` | Feedback colors (`success`, `warning`, …) |

Colors use hex strings: `#RRGGBB` or `#RRGGBBAA`.

## Built-in themes

- `nest_design::themes::light()` — id `nest-light`
- `nest_design::themes::dark()` — id `nest-dark`

Companion TOML files live in [`themes/`](../../themes/).

## Theme file format

Themes serialize as TOML or JSON. Example (`themes/nest-light.toml`):

```toml
id = "nest-light"
mode = "light"

[colors]
background = "#FFFFFF"
foreground = "#1A1A1A"
primary = "#2563EB"
# ...
```

## Related

- [Implementation plan](../plan/nest-design-theme-v1.md)
- [nest-theme](../nest-theme/README.md) — runtime loading and lifecycle
- [nest-react-theme](../nest-react-theme/README.md) — CSS/Tailwind adapter (desktop)
