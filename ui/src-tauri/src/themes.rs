//! Built-in Nest theme catalog for the Nest Theme app.
//!
//! Serves `nest-design`'s built-in [`ThemeDefinition`]s as-is (its own serde
//! shape: `ColorToken`/`ThemeId` are `#[serde(transparent)]` plain strings,
//! and token field names stay snake_case since `nest-design` doesn't rename
//! them — no transformation layer here, just exposing the source of truth).

use nest_design::{themes, ThemeDefinition};

/// Lists every built-in Nest theme with full token data.
#[tauri::command]
pub fn themes_list() -> Vec<ThemeDefinition> {
    vec![
        themes::cbre_light(),
        themes::light(),
        themes::dark(),
        themes::cursor_dark(),
    ]
}
