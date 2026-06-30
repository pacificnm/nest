//! Theme identity, mode, and full theme definition.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::tokens::{ColorTokens, RadiusTokens, SpacingTokens, StatusTokens, TypographyTokens};

/// Unique identifier for a registered theme (e.g. `"nest-light"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ThemeId(pub String);

impl ThemeId {
    /// Creates a theme id from a string value.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the underlying id string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ThemeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for ThemeId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for ThemeId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl FromStr for ThemeId {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

/// Light or dark appearance mode for a theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    /// Light background theme.
    Light,
    /// Dark background theme.
    Dark,
}

/// Complete theme definition composed of design token groups.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeDefinition {
    /// Unique theme identifier (e.g. `"nest-light"`).
    pub id: ThemeId,
    /// Light or dark appearance mode.
    pub mode: ThemeMode,
    /// Semantic color roles.
    pub colors: ColorTokens,
    /// Spacing scale in logical pixels.
    pub spacing: SpacingTokens,
    /// Border radius scale.
    pub radius: RadiusTokens,
    /// Named typography roles.
    pub typography: TypographyTokens,
    /// Status / feedback colors.
    pub status: StatusTokens,
}
