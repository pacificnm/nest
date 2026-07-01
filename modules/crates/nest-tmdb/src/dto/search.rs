use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MovieSearchResponse {
    pub results: Vec<MovieSearchResultItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MovieSearchResultItem {
    pub id: u32,
    pub title: String,
    pub overview: Option<String>,
    pub release_date: Option<String>,
}
