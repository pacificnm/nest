//! Typography design tokens.

use serde::{Deserialize, Serialize};

/// A single typography role (font family, size, line height, weight).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypographyStyle {
    /// Font family name.
    pub font_family: String,
    /// Font size in logical pixels.
    pub size: f32,
    /// Line height in logical pixels.
    pub line_height: f32,
    /// Font weight (e.g. 400, 600).
    pub weight: u16,
}

/// Named typography roles used across Nest hosts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypographyTokens {
    /// Default body text style.
    pub body: TypographyStyle,
    /// Heading text style.
    pub heading: TypographyStyle,
    /// Optional caption / small text style.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<TypographyStyle>,
    /// Optional monospace text style.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mono: Option<TypographyStyle>,
}
