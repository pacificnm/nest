use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MovieCreditsResponse {
    pub cast: Vec<CastMember>,
    pub crew: Vec<CrewMember>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CastMember {
    pub id: u32,
    pub name: String,
    pub character: Option<String>,
    pub order: Option<i32>,
    pub profile_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CrewMember {
    pub name: String,
    pub job: Option<String>,
}
