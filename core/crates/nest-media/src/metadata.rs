//! Metadata search and fetch types.

use crate::external::ExternalIds;
use crate::id::ExternalMediaId;
use crate::movie::{Movie, PersonCredit};
use crate::tracks::MediaTracks;

/// Query for searching movies in a metadata provider.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MovieSearchQuery {
    /// Search text.
    pub query: String,
    /// Optional release year filter.
    pub year: Option<u16>,
}

impl MovieSearchQuery {
    /// Creates a movie search query.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            year: None,
        }
    }

    /// Sets an optional release year filter.
    pub fn with_year(mut self, year: u16) -> Self {
        self.year = Some(year);
        self
    }
}

/// One movie search hit from a metadata provider.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MovieSearchResult {
    /// Provider media id.
    pub external_id: ExternalMediaId,
    /// Display title.
    pub title: String,
    /// Release year.
    pub year: Option<u16>,
    /// Plot summary snippet.
    pub summary: Option<String>,
}

/// Provider-normalized movie metadata before persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MovieMetadata {
    /// Provider media id.
    pub external_id: ExternalMediaId,
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
    /// Stream track metadata when known from provider.
    pub tracks: MediaTracks,
    /// External provider ids.
    pub external_ids: ExternalIds,
}

impl MovieMetadata {
    /// Maps provider metadata into a persisted movie with a Nest id.
    pub fn into_movie(self, id: crate::id::MediaId) -> Movie {
        Movie {
            id,
            title: self.title,
            original_title: self.original_title,
            sort_title: self.sort_title,
            year: self.year,
            runtime_seconds: self.runtime_seconds,
            rating: self.rating,
            summary: self.summary,
            genres: self.genres,
            cast: self.cast,
            crew: self.crew,
            artwork: Vec::new(),
            tracks: self.tracks,
            external_ids: self.external_ids,
        }
    }
}
