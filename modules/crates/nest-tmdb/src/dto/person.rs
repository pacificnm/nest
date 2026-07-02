use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PersonDetailsResponse {
    pub id: u32,
    pub name: String,
    pub biography: Option<String>,
    pub birthday: Option<String>,
    pub deathday: Option<String>,
    pub place_of_birth: Option<String>,
    pub profile_path: Option<String>,
    pub known_for_department: Option<String>,
    pub gender: Option<i32>,
    pub also_known_as: Option<Vec<String>>,
}
