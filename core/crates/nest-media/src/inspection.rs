//! Media inspection input and results.

use crate::tracks::MediaTracks;

/// Input reference for media inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MediaInput {
    /// Local filesystem media file.
    LocalPath(String),
}

/// Result of inspecting a media file.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MediaInspection {
    /// Detected stream tracks.
    pub tracks: MediaTracks,
    /// Duration in seconds.
    pub duration_seconds: Option<u32>,
    /// Container format name.
    pub container: Option<String>,
}

impl MediaInspection {
    /// Creates an inspection result with tracks.
    pub fn new(tracks: MediaTracks) -> Self {
        Self {
            tracks,
            duration_seconds: None,
            container: None,
        }
    }
}
