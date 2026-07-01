//! Movie domain model.

use crate::artwork::Artwork;
use crate::external::ExternalIds;
use crate::id::MediaId;
use crate::tracks::MediaTracks;

/// Cast or crew credit.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PersonCredit {
    /// Person name.
    pub name: String,
    /// Role or job title.
    pub role: String,
    /// Character name for cast entries.
    pub character: Option<String>,
}

impl PersonCredit {
    /// Creates a cast or crew credit.
    pub fn new(
        name: impl Into<String>,
        role: impl Into<String>,
        character: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            role: role.into(),
            character,
        }
    }
}

/// Full movie metadata stored in a media library.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Movie {
    /// Nest media id.
    pub id: MediaId,
    /// Display title.
    pub title: String,
    /// Original release title.
    pub original_title: Option<String>,
    /// Sort title for library ordering.
    pub sort_title: Option<String>,
    /// Release year.
    pub year: Option<u16>,
    /// Runtime in seconds.
    pub runtime_seconds: Option<u32>,
    /// Content rating label.
    pub rating: Option<String>,
    /// Plot summary.
    pub summary: Option<String>,
    /// Genre labels.
    pub genres: Vec<String>,
    /// Cast credits.
    pub cast: Vec<PersonCredit>,
    /// Crew credits.
    pub crew: Vec<PersonCredit>,
    /// Artwork assets.
    pub artwork: Vec<Artwork>,
    /// Stream track metadata.
    pub tracks: MediaTracks,
    /// External provider ids.
    pub external_ids: ExternalIds,
}

impl Movie {
    /// Creates a movie with required fields.
    pub fn new(id: MediaId, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            original_title: None,
            sort_title: None,
            year: None,
            runtime_seconds: None,
            rating: None,
            summary: None,
            genres: Vec::new(),
            cast: Vec::new(),
            crew: Vec::new(),
            artwork: Vec::new(),
            tracks: MediaTracks::new(),
            external_ids: ExternalIds::new(),
        }
    }
}
