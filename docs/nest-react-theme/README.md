# nest-react-theme

React and Tailwind adapter for [Nest design tokens](../nest-design/README.md).

**Part of the desktop frontend platform:** **Tauri + React + TypeScript + Tailwind**. See [architecture](../architecture.md#desktop-frontend-platform).

**Crate path:** [`core/crates/nest-react-theme`](../../core/crates/nest-react-theme)

**Status:** Implemented (v1 CSS adapter + Tailwind preset JSON).

## Role

`nest-design` defines semantic tokens (`background`, `primary`, spacing scale, …). `nest-theme` loads and manages themes at runtime in Rust. **`nest-react-theme`** bridges active theme data to the React webview:

1. **CSS custom properties** — `--nest-color-background`, `--nest-spacing-md`, …
2. **Tailwind preset** — utilities like `bg-nest-background`, `text-nest-primary`

Tailwind is the **utility layer**; `nest-design` remains the **design system source of truth**.

## Usage

```ts
// tailwind.config.ts — import preset JSON from app build or copy from nest-react-theme docs
import nestPreset from "./nest-tailwind-preset.json";

export default {
  presets: [nestPreset],
  content: ["./src/**/*.{ts,tsx}"],
};
```

Rust: `nest_react_theme::tailwind_preset_json()` returns the preset document.

```tsx
// Apply theme class on root; vars from nest_theme_css invoke or startup script
<div className="bg-nest-background text-nest-foreground">
  ...
</div>
```

## ThemeAdapter

`ReactThemeAdapter` implements `ThemeAdapter<CssTheme>` from `nest-theme`, producing a serializable CSS var map for IPC to the webview (see `nest_tauri::commands::nest_theme_css`).

## Related

- [nest-design](../nest-design/README.md) — token schema
- [nest-theme](../nest-theme/README.md) — runtime theme service
- [nest-tauri](../nest-tauri/README.md) — desktop host
