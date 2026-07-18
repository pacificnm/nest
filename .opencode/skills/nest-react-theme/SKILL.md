---
name: nest-react-theme
description: React and Tailwind adapter for Nest design tokens.
---

# Skill: nest-react-theme
This skill documents the `nest-react-theme` package, a React+Tailwind bridge to the Rust `nest-theme` runtime service.

## Purpose
Provides CSS custom properties and a Tailwind preset that expose tokens from `nest-design` into web components. The Rust side exposes a JSON representation via `nest_react_theme::tailwind_preset_json()`.

## How to Use
1. Import the preset in your Tailwind config:
```ts
import nestPreset from './nest-tailwind-preset.json';
export default { presets: [nestPreset], content: ['./src/**/*.{ts,tsx}'] };
```
2. Wrap your app root with a class using the theme utilities (e.g., `bg-nest-background`).
3. On the Rust side, call `nest_react_theme::tailwind_preset_json()` to obtain the preset file.
4. If needed, use `ReactThemeAdapter` from `nest-react-theme` to send CSS variable maps to the webview via the Tauri IPC command `nest_theme_css`.

## Related Resources
- [nest-design](../nest-design/README.md)
- [nest-theme](../nest-theme/README.md)
- [nest-tauri](../nest-tauri/README.md)
