//! Maps [`nest_design::ThemeDefinition`] tokens to CSS custom properties and a
//! Tailwind preset for React + Tauri desktop apps.

#![deny(missing_docs)]

mod adapter;
mod css;
mod tailwind;

pub use adapter::ReactThemeAdapter;
pub use css::CssTheme;
pub use tailwind::tailwind_preset_json;
