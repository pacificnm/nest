//! Library configuration.

use std::fmt;

/// Identifier for a configured media library.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct LibraryId(pub String);

impl LibraryId {
    /// Creates a library id.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the id string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LibraryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Configuration for scanning one media library.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MediaLibraryConfig {
    /// Library identifier.
    pub id: LibraryId,
    /// Root directories relative to [`nest_file::FileService`] scope.
    pub roots: Vec<String>,
    /// Video file extensions without leading dots.
    pub video_extensions: Vec<String>,
    /// Whether to follow directory symlinks while scanning.
    pub follow_symlinks: bool,
}

impl MediaLibraryConfig {
    /// Creates a library config with common video extensions.
    pub fn new(id: impl Into<String>, roots: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            id: LibraryId::new(id),
            roots: roots.into_iter().map(Into::into).collect(),
            video_extensions: default_video_extensions(),
            follow_symlinks: false,
        }
    }

    /// Overrides the video extension list.
    pub fn with_video_extensions(
        mut self,
        extensions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.video_extensions = extensions
            .into_iter()
            .map(|ext| ext.into().trim_start_matches('.').to_ascii_lowercase())
            .collect();
        self
    }
}

fn default_video_extensions() -> Vec<String> {
    vec![
        "mkv".into(),
        "mp4".into(),
        "avi".into(),
        "mov".into(),
        "m4v".into(),
        "wmv".into(),
    ]
}
