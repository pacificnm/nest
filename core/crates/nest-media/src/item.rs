//! Cross-kind media item summary.

use crate::artwork::Artwork;
use crate::external::ExternalIds;
use crate::id::MediaId;
use crate::kind::MediaKind;
use crate::tracks::MediaTracks;

/// Lightweight media summary for list views.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MediaItem {
    /// Nest media id.
    pub id: MediaId,
    /// Media classification.
    pub kind: MediaKind,
    /// Display title.
    pub title: String,
    /// Sort title for library ordering.
    pub sort_title: Option<String>,
    /// Release year.
    pub year: Option<u16>,
    /// Runtime in seconds.
    pub runtime_seconds: Option<u32>,
    /// Plot summary.
    pub summary: Option<String>,
    /// Artwork assets.
    pub artwork: Vec<Artwork>,
    /// Stream track metadata.
    pub tracks: MediaTracks,
    /// External provider ids.
    pub external_ids: ExternalIds,
}

impl MediaItem {
    /// Creates a media item summary from a movie.
    pub fn from_movie(movie: &crate::movie::Movie) -> Self {
        Self {
            id: movie.id.clone(),
            kind: MediaKind::Movie,
            title: movie.title.clone(),
            sort_title: movie.sort_title.clone(),
            year: movie.year,
            runtime_seconds: movie.runtime_seconds,
            summary: movie.summary.clone(),
            artwork: movie.artwork.clone(),
            tracks: movie.tracks.clone(),
            external_ids: movie.external_ids.clone(),
        }
    }
}
