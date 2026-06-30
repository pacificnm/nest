//! Color design tokens.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A platform-neutral color token stored as a hex string (`#RRGGBB` or `#RRGGBBAA`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ColorToken(pub String);

impl ColorToken {
    /// Creates a color token after validating the hex format.
    pub fn new(value: impl Into<String>) -> Result<Self, ColorParseError> {
        let value = value.into();
        validate_hex_color(&value)?;
        Ok(Self(value))
    }

    /// Returns the underlying hex string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the token's hex format.
    pub fn validate(&self) -> Result<(), ColorParseError> {
        validate_hex_color(&self.0)
    }
}

impl fmt::Display for ColorToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for ColorToken {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// Semantic color roles used across Nest hosts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorTokens {
    /// Main application background.
    pub background: ColorToken,
    /// Primary text and icon color.
    pub foreground: ColorToken,
    /// Primary brand / action color.
    pub primary: ColorToken,
    /// Secondary UI color.
    pub secondary: ColorToken,
    /// Border and divider color.
    pub border: ColorToken,
    /// Elevated surface / card background.
    pub surface: ColorToken,
    /// Optional accent highlight color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent: Option<ColorToken>,
    /// Optional muted / disabled text color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub muted: Option<ColorToken>,
}

/// Error returned when a color string is not valid hex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorParseError {
    /// The invalid input value.
    pub value: String,
}

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid color token {:?}: expected #RRGGBB or #RRGGBBAA",
            self.value
        )
    }
}

impl std::error::Error for ColorParseError {}

fn validate_hex_color(value: &str) -> Result<(), ColorParseError> {
    let hex = value.strip_prefix('#').ok_or_else(|| ColorParseError {
        value: value.to_string(),
    })?;

    let valid_len = matches!(hex.len(), 6 | 8);
    let all_hex = hex.chars().all(|c| c.is_ascii_hexdigit());

    if valid_len && all_hex {
        Ok(())
    } else {
        Err(ColorParseError {
            value: value.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_six_and_eight_digit_hex() {
        assert!(ColorToken::new("#112233").is_ok());
        assert!(ColorToken::new("#11223344").is_ok());
    }

    #[test]
    fn rejects_invalid_hex() {
        assert!(ColorToken::new("112233").is_err());
        assert!(ColorToken::new("#GGGGGG").is_err());
        assert!(ColorToken::new("#12345").is_err());
    }
}
