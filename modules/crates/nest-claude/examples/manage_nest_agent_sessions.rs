//! Live API smoke test for the Sessions API
//! (`ClaudeClient::create_session`/`list_sessions`/`get_session`/
//! `update_session`/`archive_session`/`delete_session`).
//!
//! A session requires an `environment_id`, and the Environments API isn't
//! wrapped by this crate yet — this example makes two small raw HTTP calls
//! (`GET`/`POST /v1/environments`) directly through `nest_http_client`, just
//! enough to find-or-create one reusable environment named "nest-desktop",
//! rather than adding a whole `environments.rs` module for this pass.
//!
//! Requires `ANTHROPIC_API_KEY` in the environment, and requires the "Nest
//! Framework Agent" to already exist (run the `create_nest_agent` example
//! first — see `docs/plan/nest-claude-v1.md` § Live resources for its id).
//! No messages are sent to the session (that's Session Events, not yet
//! covered), so it never leaves `idle` and no model inference is billed.
//!
//! ```bash
//! cargo run --example manage_nest_agent_sessions -p nest-claude
//! ```

use nest_claude::{
    ClaudeClient, ClaudeConfig, CreateSessionRequest, ListSessionsParams, SessionAgentRef,
    UpdateSessionRequest,
};
use nest_http_client::{HttpClientConfig, HttpClientService};

const NEST_AGENT_ID: &str = "agent_01DwppBhxh6j4aKQmFbPhJ5H";
const ENVIRONMENT_NAME: &str = "nest-desktop";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClaudeConfig::from_env()?;
    let client = ClaudeClient::new(config.clone())?;

    let raw_http = HttpClientService::new(
        HttpClientConfig::default()
            .with_default_header("x-api-key", config.api_key.clone())
            .with_default_header("anthropic-version", config.anthropic_version.clone())
            .with_default_header("anthropic-beta", "managed-agents-2026-04-01"),
    )?;
    let environments_url = format!("{}/v1/environments", config.base_url);

    let environment_id = find_or_create_environment(&raw_http, &environments_url).await?;
    println!("Using environment: {environment_id}");

    // Create.
    let session = client
        .create_session(
            CreateSessionRequest::new(SessionAgentRef::id(NEST_AGENT_ID), environment_id.clone())
                .title("nest-claude Sessions API smoke test"),
        )
        .await?;
    println!("\nCreated session:");
    println!("  id:     {}", session.id);
    println!("  status: {:?}", session.status);

    // Get.
    let fetched = client.get_session(&session.id).await?;
    println!("\nGet Session round-trip:");
    println!("  title:  {:?}", fetched.title);

    // List.
    let page = client
        .list_sessions(ListSessionsParams::new().limit(10))
        .await?;
    println!(
        "\n{} session(s) on this account (first page):",
        page.data.len()
    );
    for listed in &page.data {
        println!("  {} — {:?} ({:?})", listed.id, listed.title, listed.status);
    }

    // Update.
    let updated = client
        .update_session(
            &session.id,
            UpdateSessionRequest::new().title("nest-claude Sessions API smoke test (renamed)"),
        )
        .await?;
    println!("\nUpdated session title: {:?}", updated.title);

    // Archive.
    let archived = client.archive_session(&session.id).await?;
    println!("\nArchived session at: {:?}", archived.archived_at);

    // Delete.
    let deleted = client.delete_session(&session.id).await?;
    println!("Deleted session: {}", deleted.id);

    Ok(())
}

/// Finds an existing environment named [`ENVIRONMENT_NAME`], or creates one.
///
/// Raw HTTP, not a crate method — the Environments API isn't wrapped yet.
async fn find_or_create_environment(
    http: &HttpClientService,
    environments_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let list: serde_json::Value = http.get_json(environments_url).await?;
    if let Some(existing) = list["data"].as_array().and_then(|envs| {
        envs.iter()
            .find(|env| env["name"] == ENVIRONMENT_NAME)
            .and_then(|env| env["id"].as_str())
    }) {
        return Ok(existing.to_string());
    }

    let created: serde_json::Value = http
        .post_json(
            environments_url,
            &serde_json::json!({
                "name": ENVIRONMENT_NAME,
                "config": {
                    "type": "cloud",
                    "networking": { "type": "unrestricted" }
                }
            }),
        )
        .await?;
    let id = created["id"]
        .as_str()
        .ok_or("environment creation response missing \"id\"")?;
    Ok(id.to_string())
}
