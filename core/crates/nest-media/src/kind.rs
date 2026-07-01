//! Media kind classification.

/// High-level media classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MediaKind {
    /// Feature film.
    Movie,
    /// Television series.
    TvShow,
    /// Television season.
    Season,
    /// Television episode.
    Episode,
    /// Home video recording.
    HomeVideo,
    /// Unknown or unclassified media.
    #[default]
    Unknown,
}
