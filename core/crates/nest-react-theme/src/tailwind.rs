//! Tailwind CSS preset referencing Nest CSS variables.

/// Returns a Tailwind v3 preset JSON document that maps `nest-*` utilities to
/// `--nest-*` CSS custom properties produced by [`crate::CssTheme`].
pub fn tailwind_preset_json() -> &'static str {
    r#"{
  "theme": {
    "extend": {
      "colors": {
        "nest-background": "var(--nest-color-background)",
        "nest-foreground": "var(--nest-color-foreground)",
        "nest-primary": "var(--nest-color-primary)",
        "nest-secondary": "var(--nest-color-secondary)",
        "nest-border": "var(--nest-color-border)",
        "nest-surface": "var(--nest-color-surface)",
        "nest-accent": "var(--nest-color-accent)",
        "nest-muted": "var(--nest-color-muted)",
        "nest-success": "var(--nest-color-success)",
        "nest-warning": "var(--nest-color-warning)",
        "nest-error": "var(--nest-color-error)",
        "nest-info": "var(--nest-color-info)"
      },
      "spacing": {
        "nest-xs": "var(--nest-spacing-xs)",
        "nest-sm": "var(--nest-spacing-sm)",
        "nest-md": "var(--nest-spacing-md)",
        "nest-lg": "var(--nest-spacing-lg)",
        "nest-xl": "var(--nest-spacing-xl)",
        "nest-xxl": "var(--nest-spacing-xxl)"
      },
      "borderRadius": {
        "nest-sm": "var(--nest-radius-sm)",
        "nest-md": "var(--nest-radius-md)",
        "nest-lg": "var(--nest-radius-lg)",
        "nest-full": "var(--nest-radius-full)"
      }
    }
  }
}"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_is_valid_json() {
        let _: serde_json::Value = serde_json::from_str(tailwind_preset_json()).unwrap();
    }
}
