use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MovieExternalIdsResponse {
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<u32>,
}
