//! Internal FFprobe JSON types.

pub(crate) mod format;
pub(crate) mod stream;

use format::FormatSection;
use stream::StreamSection;

/// Parsed FFprobe JSON output.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct FfprobeOutput {
    #[serde(default)]
    pub streams: Vec<StreamSection>,
    pub format: Option<FormatSection>,
}
