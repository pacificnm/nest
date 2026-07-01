//! Artwork models.

/// Kind of artwork asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ArtworkKind {
    /// Poster image.
    Poster,
    /// Backdrop image.
    Backdrop,
    /// Logo image.
    Logo,
    /// Thumbnail image.
    Thumbnail,
    /// Episode or scene still.
    Still,
}

/// Source location for artwork.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ArtworkSource {
    /// Local filesystem path.
    LocalPath(String),
    /// Remote HTTP(S) URL.
    RemoteUrl(String),
}

/// Artwork metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Artwork {
    /// Artwork type.
    pub kind: ArtworkKind,
    /// Source location.
    pub source: ArtworkSource,
    /// Pixel width, if known.
    pub width: Option<u32>,
    /// Pixel height, if known.
    pub height: Option<u32>,
}
