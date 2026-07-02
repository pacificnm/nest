//! Shared AI request and response types.

use serde::{Deserialize, Serialize};

/// Chat role for multi-turn prompts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    /// System instruction.
    System,
    /// User input.
    User,
    /// Model output.
    Assistant,
}

/// One chat message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message role.
    pub role: ChatRole,
    /// Message text.
    pub content: String,
}

impl ChatMessage {
    /// Creates a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
        }
    }

    /// Creates a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
        }
    }

    /// Creates an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
        }
    }
}

/// Desired response shape from the provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormat {
    /// Plain text.
    Text,
    /// JSON object (provider-specific enforcement).
    Json,
}

/// Provider-agnostic completion request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model id; providers use their default when omitted.
    pub model: Option<String>,
    /// Chat messages in order.
    pub messages: Vec<ChatMessage>,
    /// Optional response format hint.
    pub format: Option<ResponseFormat>,
}

impl CompletionRequest {
    /// Creates a single user-turn request.
    pub fn user_message(content: impl Into<String>) -> Self {
        Self {
            model: None,
            messages: vec![ChatMessage::user(content)],
            format: None,
        }
    }

    /// Sets the model id.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Requests JSON output when supported.
    pub fn with_json_format(mut self) -> Self {
        self.format = Some(ResponseFormat::Json);
        self
    }
}

/// Provider-agnostic completion response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Model that produced the response.
    pub model: String,
    /// Assistant text content.
    pub content: String,
    /// Whether the provider marked the response complete.
    pub done: bool,
}
