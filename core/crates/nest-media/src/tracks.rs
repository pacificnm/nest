//! Audio, video, and subtitle track models.

/// HDR format for a video track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HdrFormat {
    /// HDR10.
    Hdr10,
    /// Dolby Vision.
    DolbyVision,
    /// HLG.
    Hlg,
}

/// Container for all media tracks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MediaTracks {
    /// Video tracks.
    pub video: Vec<VideoTrack>,
    /// Audio tracks.
    pub audio: Vec<AudioTrack>,
    /// Subtitle tracks.
    pub subtitles: Vec<SubtitleTrack>,
}

impl MediaTracks {
    /// Creates empty track collections.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Video stream metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VideoTrack {
    /// Video codec name.
    pub codec: Option<String>,
    /// Frame width in pixels.
    pub width: Option<u32>,
    /// Frame height in pixels.
    pub height: Option<u32>,
    /// Bitrate in bits per second.
    pub bitrate: Option<u64>,
    /// HDR format, if applicable.
    pub hdr: Option<HdrFormat>,
}

/// Audio stream metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AudioTrack {
    /// Audio codec name.
    pub codec: Option<String>,
    /// Channel layout (e.g. `5.1`).
    pub channels: Option<String>,
    /// Language tag.
    pub language: Option<String>,
    /// Display title.
    pub title: Option<String>,
}

/// Subtitle stream metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubtitleTrack {
    /// Subtitle codec or format.
    pub codec: Option<String>,
    /// Language tag.
    pub language: Option<String>,
    /// Display title.
    pub title: Option<String>,
    /// Forced subtitle track.
    pub forced: bool,
    /// Default subtitle track.
    pub is_default: bool,
}
