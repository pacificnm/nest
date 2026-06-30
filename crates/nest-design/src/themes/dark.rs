//! Built-in Nest dark theme.

use crate::theme::{ThemeDefinition, ThemeId, ThemeMode};
use crate::tokens::{
    ColorToken, ColorTokens, RadiusTokens, SpacingTokens, StatusTokens, TypographyStyle,
    TypographyTokens,
};

/// Returns the default Nest dark theme.
pub fn dark() -> ThemeDefinition {
    ThemeDefinition {
        id: ThemeId::new("nest-dark"),
        mode: ThemeMode::Dark,
        colors: ColorTokens {
            background: color("#0F172A"),
            foreground: color("#F1F5F9"),
            primary: color("#3B82F6"),
            secondary: color("#94A3B8"),
            border: color("#334155"),
            surface: color("#1E293B"),
            accent: Some(color("#A78BFA")),
            muted: Some(color("#64748B")),
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
            success: color("#22C55E"),
            warning: color("#F59E0B"),
            error: color("#EF4444"),
            info: color("#3B82F6"),
        },
    }
}

fn color(value: &str) -> ColorToken {
    ColorToken::new(value).expect("built-in theme colors are valid")
}
