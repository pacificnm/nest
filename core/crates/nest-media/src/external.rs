//! External provider identifiers.

/// External provider ids attached to media entities.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExternalIds {
    /// TMDB id.
    pub tmdb_id: Option<String>,
    /// IMDb id.
    pub imdb_id: Option<String>,
    /// TVDB id.
    pub tvdb_id: Option<String>,
}

impl ExternalIds {
    /// Creates empty external ids.
    pub fn new() -> Self {
        Self::default()
    }
}
