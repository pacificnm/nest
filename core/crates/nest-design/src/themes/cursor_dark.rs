//! Built-in Cursor-inspired dark theme.
//!
//! Mirrors the muted, low-contrast dark palette used by Cursor / VS Code so
//! IDE-style Nest apps (Kiwi) get a familiar developer look. See
//! `apps/kiwi/docs/theming/cursor-dark.md` for the source palette.

use crate::theme::{ThemeDefinition, ThemeId, ThemeMode};
use crate::tokens::{
    ColorToken, ColorTokens, RadiusTokens, SpacingTokens, StatusTokens, TypographyStyle,
    TypographyTokens,
};

/// Returns the Cursor-inspired dark theme (`cursor-dark`).
pub fn cursor_dark() -> ThemeDefinition {
    ThemeDefinition {
        id: ThemeId::new("cursor-dark"),
        mode: ThemeMode::Dark,
        colors: ColorTokens {
            // Editor canvas — near-black slate.
            background: color("#1B1F23"),
            // Soft off-white body text (avoids pure white glare).
            foreground: color("#CCCCCC"),
            // Cursor blue for primary actions and highlights.
            primary: color("#4F8EF7"),
            // Muted blue-gray for secondary chrome.
            secondary: color("#9DA5B4"),
            // Low-contrast panel dividers.
            border: color("#313842"),
            // Sidebar / panel surface, one step above the editor.
            surface: color("#252B32"),
            // Accent matches the primary Cursor blue.
            accent: Some(color("#4F8EF7")),
            // Dim gray for secondary/disabled text.
            muted: Some(color("#6E7681")),
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
            md: 6.0,
            lg: 8.0,
            full: Some(9999.0),
        },
        typography: TypographyTokens {
            body: TypographyStyle {
                font_family: "Inter, system-ui, -apple-system, sans-serif".to_string(),
                size: 13.0,
                line_height: 18.0,
                weight: 400,
            },
            heading: TypographyStyle {
                font_family: "Inter, system-ui, -apple-system, sans-serif".to_string(),
                size: 18.0,
                line_height: 24.0,
                weight: 500,
            },
            caption: Some(TypographyStyle {
                font_family: "Inter, system-ui, -apple-system, sans-serif".to_string(),
                size: 11.0,
                line_height: 14.0,
                weight: 400,
            }),
            mono: Some(TypographyStyle {
                font_family: "JetBrains Mono, Consolas, Menlo, monospace".to_string(),
                size: 13.0,
                line_height: 18.0,
                weight: 400,
            }),
        },
        status: StatusTokens {
            success: color("#3FB950"),
            warning: color("#D29922"),
            error: color("#F85149"),
            info: color("#58A6FF"),
        },
    }
}

fn color(value: &str) -> ColorToken {
    ColorToken::new(value).expect("built-in theme colors are valid")
}
