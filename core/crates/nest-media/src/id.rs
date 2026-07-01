//! Media identifiers.

use std::fmt;

/// Nest-generated media identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MediaId(pub String);

impl MediaId {
    /// Creates a new media id.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the id string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MediaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// External provider media identifier (e.g. TMDB id).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExternalMediaId(pub String);

impl ExternalMediaId {
    /// Creates a new external media id.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the id string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ExternalMediaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
