//! Claude (Anthropic) Messages API client for the Nest framework.
//!
//! `nest-claude` wraps `POST /v1/messages` — text/image content, tool use,
//! extended thinking, prompt caching, and streaming (SSE) — behind
//! [`ClaudeClient`]. It owns a dedicated [`nest_http_client::HttpClientService`]
//! rather than the app-wide shared one, since the Claude API key must not leak
//! into other modules' requests.
//!
//! # Quick start
//!
//! ```no_run
//! use nest_claude::prelude::*;
//!
//! # async fn demo() -> ClaudeResult<()> {
//! let client = ClaudeClient::new(ClaudeConfig::from_env()?)?;
//!
//! let response = client
//!     .create_message(CreateMessageRequest::new(vec![Message::user(
//!         "What is the capital of France?",
//!     )]))
//!     .await?;
//!
//! println!("{}", response.text());
//! # Ok(())
//! # }
//! ```
//!
//! ## Streaming
//!
//! ```no_run
//! use futures_util::StreamExt;
//! use nest_claude::prelude::*;
//!
//! # async fn demo(client: &ClaudeClient) -> ClaudeResult<()> {
//! let mut stream = client
//!     .stream_message(CreateMessageRequest::new(vec![Message::user("Write a haiku")]))
//!     .await?;
//!
//! while let Some(event) = stream.next().await {
//!     if let StreamEvent::ContentBlockDelta {
//!         delta: ContentDelta::TextDelta { text },
//!         ..
//!     } = event?
//!     {
//!         print!("{text}");
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## As a Nest module
//!
//! ```no_run
//! use nest_claude::{ClaudeConfig, ClaudeModule};
//! use nest_core::AppBuilder;
//!
//! let built = AppBuilder::new()
//!     .module(ClaudeModule::with_config(
//!         ClaudeConfig::builder().api_key("sk-ant-...").build().unwrap(),
//!     ))
//!     .build()
//!     .unwrap();
//!
//! let client = built.context.service::<nest_claude::ClaudeClient>().unwrap();
//! ```
//!
//! ## Endpoints covered
//!
//! - `POST /v1/messages` — non-streaming and streaming, text/image content,
//!   tool use, extended (adaptive) thinking + effort, and prompt caching
//! - `POST /v1/messages/count_tokens` — [`ClaudeClient::count_tokens`]
//! - Skills: Create/List/Get/Delete — [`ClaudeClient::create_skill`],
//!   [`ClaudeClient::list_skills`], [`ClaudeClient::get_skill`],
//!   [`ClaudeClient::delete_skill`] (skill *versions* are not yet covered)
//! - Agents (Managed Agents): Create/List/Get/Update/Archive —
//!   [`ClaudeClient::create_agent`], [`ClaudeClient::list_agents`],
//!   [`ClaudeClient::get_agent`], [`ClaudeClient::update_agent`],
//!   [`ClaudeClient::archive_agent`]
//! - Sessions (Managed Agents): Create/List/Get/Update/Delete/Archive —
//!   [`ClaudeClient::create_session`], [`ClaudeClient::list_sessions`],
//!   [`ClaudeClient::get_session`], [`ClaudeClient::update_session`],
//!   [`ClaudeClient::delete_session`], [`ClaudeClient::archive_session`]
//!   (Session Events — sending messages, streaming replies — are not yet
//!   covered; a session alone cannot yet hold a live conversation)
//!
//! Not yet covered: Files API, PDF/document content blocks, server-side tools
//! (web search, code execution), the Batches API, skill versions, Managed
//! Agents Session Events/environments/vaults, and structured outputs
//! (`output_config.format`).

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

mod agents;
mod client;
mod codes;
mod config;
mod count_tokens;
mod error;
mod module;
mod request;
mod response;
mod sessions;
mod skills;
mod stream;
mod types;
mod util;

pub mod prelude;

pub use agents::{
    Agent, AgentListPage, AgentModel, AgentModelInfo, AgentRef, AgentSkillRef, AgentTool,
    CreateAgentRequest, ListAgentsParams, McpServerDefinition, Multiagent, NamedToolConfig,
    PermissionPolicy, Speed, ToolConfig, UpdateAgentRequest,
};
pub use client::ClaudeClient;
pub use codes::{
    NEST_CLAUDE_API_ERROR, NEST_CLAUDE_AUTH_FAILED, NEST_CLAUDE_CONFIG, NEST_CLAUDE_FAILED,
    NEST_CLAUDE_INVALID_REQUEST, NEST_CLAUDE_PARSE_FAILED, NEST_CLAUDE_RATE_LIMITED,
    NEST_CLAUDE_REQUEST_FAILED, NEST_CLAUDE_SERVER_ERROR,
};
pub use config::{
    resolve_api_key, ClaudeConfig, ClaudeConfigBuilder, DEFAULT_ANTHROPIC_VERSION,
    DEFAULT_API_KEY_ENV, DEFAULT_BASE_URL, DEFAULT_MAX_TOKENS, DEFAULT_MODEL,
};
pub use count_tokens::{ContextManagementTokenInfo, CountTokensRequest, TokenCountResponse};
pub use error::{ClaudeError, ClaudeErrorKind, ClaudeResult};
pub use module::{ClaudeModule, CLAUDE_MODULE_ID};
pub use request::{
    CreateMessageRequest, Effort, Message, OutputConfig, SystemPrompt, ThinkingConfig,
    ThinkingDisplay, ToolChoice, ToolDefinition,
};
pub use response::MessageResponse;
pub use sessions::{
    CreateSessionRequest, ListSessionsParams, Session, SessionAgentRef, SessionDeleted,
    SessionListPage, SessionOrder, SessionStatus, UpdateSessionRequest,
};
pub use skills::{FileUpload, ListSkillsParams, Skill, SkillDeleted, SkillListPage, SkillSource};
pub use stream::{ApiErrorBody, ContentDelta, MessageDeltaFields, MessageStream, StreamEvent};
pub use types::{
    CacheControl, CacheTtl, ContentBlock, ImageSource, Role, StopReason, ToolResultContent, Usage,
};

pub use nest_error::{NestError, NestResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_error_converts_to_nest_error() {
        let claude_error = ClaudeError::parse("bad json");
        let nest_error: NestError = claude_error.into();
        assert_eq!(nest_error.kind(), nest_error::NestErrorKind::Validation);
        assert_eq!(nest_error.code(), Some(NEST_CLAUDE_PARSE_FAILED));
        assert_eq!(nest_error.module(), Some("nest-claude"));
    }
}
