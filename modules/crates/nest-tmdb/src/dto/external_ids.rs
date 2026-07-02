use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MovieExternalIdsResponse {
    pub imdb_id: Option<String>,
    #[allow(dead_code)]
    pub tmdb_id: Option<u32>,
}
