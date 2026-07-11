//! Request/response types for the Skills API (`/v1/skills`).
//!
//! Every Skills endpoint requires the `anthropic-beta: skills-2025-10-02`
//! header — [`ClaudeClient`](crate::ClaudeClient) sends it per-request here
//! rather than as a default header, since it does not apply to `/v1/messages`.

use serde::{Deserialize, Serialize};

use crate::util::percent_encode;

/// A single file to upload when creating a skill (e.g. `SKILL.md` and any
/// supporting files/scripts).
#[derive(Debug, Clone)]
pub struct FileUpload {
    /// The filename, including a relative directory prefix (e.g.
    /// `"my-skill/SKILL.md"`) — the API derives the skill's `directory` from
    /// the common top-level prefix across uploaded files.
    pub filename: String,
    /// The raw file bytes.
    pub content: Vec<u8>,
}

impl FileUpload {
    /// Creates a file upload from a filename and its raw content.
    pub fn new(filename: impl Into<String>, content: impl Into<Vec<u8>>) -> Self {
        Self {
            filename: filename.into(),
            content: content.into(),
        }
    }
}

/// Who created a skill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillSource {
    /// Created by a user.
    Custom,
    /// Created by Anthropic.
    Anthropic,
}

impl SkillSource {
    fn as_query_value(self) -> &'static str {
        match self {
            Self::Custom => "custom",
            Self::Anthropic => "anthropic",
        }
    }
}

/// A Claude Agent Skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique identifier for the skill.
    pub id: String,
    /// ISO 8601 timestamp of when the skill was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the skill was last updated.
    pub updated_at: String,
    /// Human-readable label, not included in the prompt sent to the model.
    pub display_title: String,
    /// The latest version identifier (a Unix epoch timestamp string).
    pub latest_version: String,
    /// Who created the skill.
    pub source: SkillSource,
}

/// One page of `GET /v1/skills`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillListPage {
    /// The skills in this page.
    pub data: Vec<Skill>,
    /// Whether more results are available via `next_page`.
    pub has_more: bool,
    /// Pagination token for the next page, when `has_more` is `true`.
    pub next_page: Option<String>,
}

/// Response from `DELETE /v1/skills/{skill_id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDeleted {
    /// The id of the deleted skill.
    pub id: String,
}

/// Query parameters for `GET /v1/skills`.
#[derive(Debug, Clone, Default)]
pub struct ListSkillsParams {
    limit: Option<u32>,
    page: Option<String>,
    source: Option<SkillSource>,
}

impl ListSkillsParams {
    /// Creates empty (default) list parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the page size (max 100, defaults to 20 server-side).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the pagination token from a previous response's `next_page`.
    pub fn page(mut self, page: impl Into<String>) -> Self {
        self.page = Some(page.into());
        self
    }

    /// Filters by who created the skill.
    pub fn source(mut self, source: SkillSource) -> Self {
        self.source = Some(source);
        self
    }

    pub(crate) fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(format!("limit={limit}"));
        }
        if let Some(page) = &self.page {
            params.push(format!("page={}", percent_encode(page)));
        }
        if let Some(source) = self.source {
            params.push(format!("source={}", source.as_query_value()));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

/// Generates a boundary string unlikely to collide with uploaded file content.
pub(crate) fn generate_boundary() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("NestClaudeBoundary{nanos:x}")
}

/// Encodes `files` as a `multipart/form-data` body, each under the `files[]`
/// field — the API rejects a bare `files` field name with `files[]: Field
/// required`, even though its own docs show `-F files=...` (singular) in the
/// example curl command.
pub(crate) fn encode_multipart_files(files: &[FileUpload], boundary: &str) -> Vec<u8> {
    let mut body = Vec::new();
    for file in files {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"files[]\"; filename=\"{}\"\r\n",
                file.filename
            )
            .as_bytes(),
        );
        body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
        body.extend_from_slice(&file.content);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_round_trips_through_serialize_and_deserialize() {
        let skill: Skill = serde_json::from_value(serde_json::json!({
            "id": "skill_01abc",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
            "display_title": "nest-core",
            "latest_version": "123",
            "source": "custom"
        }))
        .unwrap();

        let json = serde_json::to_value(&skill).unwrap();
        let reparsed: Skill = serde_json::from_value(json).unwrap();
        assert_eq!(reparsed.id, skill.id);
        assert_eq!(reparsed.source, skill.source);
    }

    #[test]
    fn query_string_is_empty_with_no_params() {
        assert_eq!(ListSkillsParams::new().to_query_string(), "");
    }

    #[test]
    fn query_string_includes_set_params() {
        let query = ListSkillsParams::new()
            .limit(50)
            .source(SkillSource::Custom)
            .page("page_abc=")
            .to_query_string();
        assert!(query.starts_with('?'));
        assert!(query.contains("limit=50"));
        assert!(query.contains("source=custom"));
        assert!(query.contains("page=page_abc%3D"));
    }

    #[test]
    fn source_serializes_lowercase() {
        assert_eq!(
            serde_json::to_value(SkillSource::Anthropic).unwrap(),
            "anthropic"
        );
    }

    #[test]
    fn multipart_body_contains_boundary_and_file_content() {
        let files = vec![FileUpload::new(
            "my-skill/SKILL.md",
            b"---\nname: my-skill\n---\nBody".to_vec(),
        )];
        let boundary = "TestBoundary";
        let body = encode_multipart_files(&files, boundary);
        let text = String::from_utf8(body).unwrap();

        assert!(text.starts_with("--TestBoundary\r\n"));
        assert!(text.contains("name=\"files[]\"; filename=\"my-skill/SKILL.md\""));
        assert!(text.contains("Body"));
        assert!(text.trim_end().ends_with("--TestBoundary--"));
    }

    #[test]
    fn generated_boundaries_are_not_all_identical() {
        let boundaries: Vec<String> = (0..8).map(|_| generate_boundary()).collect();
        assert!(boundaries.iter().any(|b| b != &boundaries[0]));
    }
}
