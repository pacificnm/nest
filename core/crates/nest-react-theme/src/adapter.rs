//! [`ThemeAdapter`] implementation for React / Tailwind hosts.

use nest_design::ThemeDefinition;
use nest_error::NestResult;
use nest_theme::ThemeAdapter;

use crate::css::CssTheme;

/// Adapts Nest design tokens to CSS variables for the webview.
pub struct ReactThemeAdapter;

impl ThemeAdapter<CssTheme> for ReactThemeAdapter {
    fn adapt(theme: &ThemeDefinition) -> NestResult<CssTheme> {
        CssTheme::from_definition(theme)
    }
}

#[cfg(test)]
mod tests {
    use nest_design::themes::dark;

    use super::*;

    #[test]
    fn adapts_builtin_dark_theme() {
        let theme = dark();
        let css = ReactThemeAdapter::adapt(&theme).unwrap();
        assert_eq!(css.id, "nest-dark");
        assert!(css.variables.contains_key("--nest-color-background"));
    }
}
