use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MovieImagesResponse {
    pub posters: Vec<ImageItem>,
    pub backdrops: Vec<ImageItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ImageItem {
    pub file_path: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}
