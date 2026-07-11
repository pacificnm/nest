//! Claude Config app — Skills/Agents viewer backed by `nest-claude`.
//!
//! Read-only for now: lists Skills and Agents from the real Claude API so
//! the Nest Desktop UI can show which skills are assigned to which agents.
//! Returns `nest_claude`'s own response types (`Skill`, `Agent`, …) directly
//! rather than a re-mapped view type — same "expose the source-of-truth
//! shape as-is" call as `themes.rs`, so the JSON field names the frontend
//! sees are `nest-claude`'s own (snake_case), not this app's usual camelCase.

use nest_claude::{Agent, ClaudeClient, ClaudeConfig, ListAgentsParams, ListSkillsParams, Skill};

use crate::claude_settings::ClaudeSettingsStore;

fn build_client(store: &ClaudeSettingsStore) -> Result<ClaudeClient, String> {
    let settings = store.get();
    let api_key = settings.api_key.trim();
    if api_key.is_empty() {
        return Err("Claude API key not configured — set it in Claude Config first.".to_string());
    }

    let config = ClaudeConfig::builder()
        .api_key(api_key)
        .build()
        .map_err(|error| error.to_string())?;
    ClaudeClient::new(config).map_err(|error| error.to_string())
}

/// Lists every Skill (both `custom` and Anthropic-provided) on the account.
#[tauri::command]
pub async fn claude_list_skills(
    store: tauri::State<'_, ClaudeSettingsStore>,
) -> Result<Vec<Skill>, String> {
    let client = build_client(&store)?;
    let page = client
        .list_skills(ListSkillsParams::new().limit(100))
        .await
        .map_err(|error| error.to_string())?;
    Ok(page.data)
}

/// Lists every non-archived Agent on the account, including their assigned skills.
#[tauri::command]
pub async fn claude_list_agents(
    store: tauri::State<'_, ClaudeSettingsStore>,
) -> Result<Vec<Agent>, String> {
    let client = build_client(&store)?;
    let page = client
        .list_agents(ListAgentsParams::new().limit(100).include_archived(false))
        .await
        .map_err(|error| error.to_string())?;
    Ok(page.data)
}
