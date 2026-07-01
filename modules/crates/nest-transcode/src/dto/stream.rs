//! FFprobe stream section.

use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct StreamSection {
    pub codec_type: Option<String>,
    pub codec_name: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub bit_rate: Option<String>,
    pub channels: Option<u32>,
    pub channel_layout: Option<String>,
    #[serde(default)]
    pub tags: HashMap<String, String>,
    pub disposition: Option<DispositionSection>,
    #[serde(default)]
    pub side_data_list: Vec<SideDataSection>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct DispositionSection {
    #[serde(default)]
    pub default: u8,
    #[serde(default)]
    pub forced: u8,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct SideDataSection {
    pub side_data_type: Option<String>,
}
