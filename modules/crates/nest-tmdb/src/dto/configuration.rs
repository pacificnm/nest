use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ConfigurationResponse {
    pub images: ImageConfiguration,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ImageConfiguration {
    pub base_url: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub poster_sizes: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub backdrop_sizes: Vec<String>,
}
