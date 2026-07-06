//! Shared design tokens and theme definitions for the Nest framework.
//!
//! `nest-design` owns the token schema. Runtime theme loading and lifecycle
//! live in `nest-theme`; platform-specific rendering conversion lives in
//! adapter crates such as `nest-react-theme`.

#![warn(missing_docs)]

pub mod theme;
pub mod themes;
pub mod tokens;

pub use theme::{ThemeDefinition, ThemeId, ThemeMode};
pub use tokens::{
    ColorParseError, ColorToken, ColorTokens, RadiusTokens, SpacingTokens, StatusTokens,
    TypographyStyle, TypographyTokens,
};

#[cfg(test)]
mod tests {
    use super::*;
    use themes::{dark, light};

    #[test]
    fn light_theme_has_expected_id() {
        let theme = light();
        assert_eq!(theme.id.as_str(), "nest-light");
        assert_eq!(theme.mode, ThemeMode::Light);
    }

    #[test]
    fn dark_theme_has_expected_id() {
        let theme = dark();
        assert_eq!(theme.id.as_str(), "nest-dark");
        assert_eq!(theme.mode, ThemeMode::Dark);
    }

    #[test]
    fn serde_round_trip_light() {
        let theme = light();
        let json = serde_json::to_string(&theme).unwrap();
        let parsed: ThemeDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, theme);
    }

    #[test]
    fn serde_round_trip_dark() {
        let theme = dark();
        let toml_str = toml::to_string(&theme).unwrap();
        let parsed: ThemeDefinition = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed, theme);
    }

    #[test]
    fn all_builtin_colors_validate() {
        for theme in [light(), dark()] {
            theme.colors.background.validate().unwrap();
            theme.colors.foreground.validate().unwrap();
            theme.colors.primary.validate().unwrap();
            theme.status.success.validate().unwrap();
        }
    }
}
