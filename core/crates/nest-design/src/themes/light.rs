//! Built-in Nest light theme.

use crate::theme::{ThemeDefinition, ThemeId, ThemeMode};
use crate::tokens::{
    ColorToken, ColorTokens, RadiusTokens, SpacingTokens, StatusTokens, TypographyStyle,
    TypographyTokens,
};

/// Returns the default Nest light theme.
pub fn light() -> ThemeDefinition {
    ThemeDefinition {
        id: ThemeId::new("nest-light"),
        mode: ThemeMode::Light,
        colors: ColorTokens {
            background: color("#FFFFFF"),
            foreground: color("#1A1A1A"),
            primary: color("#2563EB"),
            secondary: color("#64748B"),
            border: color("#E2E8F0"),
            surface: color("#F8FAFC"),
            accent: Some(color("#7C3AED")),
            muted: Some(color("#94A3B8")),
        },
        spacing: SpacingTokens {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
            xxl: Some(48.0),
        },
        radius: RadiusTokens {
            sm: 4.0,
            md: 8.0,
            lg: 12.0,
            full: Some(9999.0),
        },
        typography: TypographyTokens {
            body: TypographyStyle {
                font_family: "Inter".to_string(),
                size: 14.0,
                line_height: 20.0,
                weight: 400,
            },
            heading: TypographyStyle {
                font_family: "Inter".to_string(),
                size: 20.0,
                line_height: 28.0,
                weight: 600,
            },
            caption: Some(TypographyStyle {
                font_family: "Inter".to_string(),
                size: 12.0,
                line_height: 16.0,
                weight: 400,
            }),
            mono: Some(TypographyStyle {
                font_family: "JetBrains Mono".to_string(),
                size: 13.0,
                line_height: 18.0,
                weight: 400,
            }),
        },
        status: StatusTokens {
            success: color("#16A34A"),
            warning: color("#D97706"),
            error: color("#DC2626"),
            info: color("#2563EB"),
        },
    }
}

fn color(value: &str) -> ColorToken {
    ColorToken::new(value).expect("built-in theme colors are valid")
}
