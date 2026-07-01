//! FFprobe format section.

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct FormatSection {
    pub format_name: Option<String>,
    pub duration: Option<String>,
}
