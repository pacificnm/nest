use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MovieDetailsResponse {
    pub id: u32,
    pub title: String,
    pub original_title: Option<String>,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub runtime: Option<u32>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub genres: Vec<GenreItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GenreItem {
    pub name: String,
}
