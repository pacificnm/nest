//! CSS custom property output for webview theming.

use std::collections::BTreeMap;

use nest_design::{ThemeDefinition, ThemeMode};
use nest_error::NestResult;

/// Theme data serialized for the React layer (`:root` CSS variables).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CssTheme {
    /// Active theme id (e.g. `nest-light`).
    pub id: String,
    /// Light or dark mode.
    pub mode: ThemeMode,
    /// CSS custom properties (`--nest-color-background`, …).
    pub variables: BTreeMap<String, String>,
}

impl CssTheme {
    /// Builds a `:root { … }` block for injection into the webview.
    pub fn to_root_block(&self) -> String {
        let mut lines = Vec::with_capacity(self.variables.len() + 2);
        lines.push(":root {".to_string());
        for (name, value) in &self.variables {
            lines.push(format!("  {name}: {value};"));
        }
        lines.push("}".to_string());
        lines.join("\n")
    }

    /// Builds CSS variables from a [`ThemeDefinition`].
    pub fn from_definition(theme: &ThemeDefinition) -> NestResult<Self> {
        let mut variables = BTreeMap::new();

        let colors = &theme.colors;
        insert_color(&mut variables, "background", colors.background.as_str());
        insert_color(&mut variables, "foreground", colors.foreground.as_str());
        insert_color(&mut variables, "primary", colors.primary.as_str());
        insert_color(&mut variables, "secondary", colors.secondary.as_str());
        insert_color(&mut variables, "border", colors.border.as_str());
        insert_color(&mut variables, "surface", colors.surface.as_str());
        if let Some(accent) = &colors.accent {
            insert_color(&mut variables, "accent", accent.as_str());
        }
        if let Some(muted) = &colors.muted {
            insert_color(&mut variables, "muted", muted.as_str());
        }

        let status = &theme.status;
        insert_color(&mut variables, "success", status.success.as_str());
        insert_color(&mut variables, "warning", status.warning.as_str());
        insert_color(&mut variables, "error", status.error.as_str());
        insert_color(&mut variables, "info", status.info.as_str());

        let spacing = &theme.spacing;
        insert_px(&mut variables, "spacing-xs", spacing.xs);
        insert_px(&mut variables, "spacing-sm", spacing.sm);
        insert_px(&mut variables, "spacing-md", spacing.md);
        insert_px(&mut variables, "spacing-lg", spacing.lg);
        insert_px(&mut variables, "spacing-xl", spacing.xl);
        if let Some(xxl) = spacing.xxl {
            insert_px(&mut variables, "spacing-xxl", xxl);
        }

        let radius = &theme.radius;
        insert_px(&mut variables, "radius-sm", radius.sm);
        insert_px(&mut variables, "radius-md", radius.md);
        insert_px(&mut variables, "radius-lg", radius.lg);
        if let Some(full) = radius.full {
            insert_px(&mut variables, "radius-full", full);
        }

        Ok(Self {
            id: theme.id.as_str().to_string(),
            mode: theme.mode,
            variables,
        })
    }
}

fn insert_color(vars: &mut BTreeMap<String, String>, role: &str, value: &str) {
    vars.insert(format!("--nest-color-{role}"), value.to_string());
}

fn insert_px(vars: &mut BTreeMap<String, String>, role: &str, value: f32) {
    vars.insert(format!("--nest-{role}"), format!("{value}px"));
}

#[cfg(test)]
mod tests {
    use nest_design::themes::light;

    use super::*;

    #[test]
    fn root_block_contains_background_var() {
        let theme = light();
        let css = CssTheme::from_definition(&theme).unwrap();
        let block = css.to_root_block();
        assert!(block.contains("--nest-color-background:"));
        assert!(block.starts_with(":root"));
    }
}
