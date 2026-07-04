//! Shared AI request and response types.

use serde::{Deserialize, Serialize};

use crate::tools::{ToolCall, ToolDefinition};

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
    /// Tool execution result returned to the model.
    Tool,
}

/// One chat message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message role.
    pub role: ChatRole,
    /// Message text.
    #[serde(default)]
    pub content: String,
    /// Tool name when [`ChatRole::Tool`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Tool invocations when [`ChatRole::Assistant`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl ChatMessage {
    /// Creates a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
            tool_name: None,
            tool_calls: None,
        }
    }

    /// Creates a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
            tool_name: None,
            tool_calls: None,
        }
    }

    /// Creates an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
            tool_name: None,
            tool_calls: None,
        }
    }

    /// Creates an assistant message that requests tool calls.
    pub fn assistant_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: String::new(),
            tool_name: None,
            tool_calls: Some(tool_calls),
        }
    }

    /// Creates a tool result message.
    pub fn tool_result(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Tool,
            content: content.into(),
            tool_name: Some(name.into()),
            tool_calls: None,
        }
    }

    /// Returns true when the assistant requested one or more tools.
    pub fn has_tool_calls(&self) -> bool {
        self.tool_calls
            .as_ref()
            .is_some_and(|calls| !calls.is_empty())
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model id; providers use their default when omitted.
    pub model: Option<String>,
    /// Chat messages in order.
    pub messages: Vec<ChatMessage>,
    /// Optional response format hint.
    pub format: Option<ResponseFormat>,
    /// Tool definitions available to the model.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolDefinition>,
}

impl CompletionRequest {
    /// Creates a single user-turn request.
    pub fn user_message(content: impl Into<String>) -> Self {
        Self {
            model: None,
            messages: vec![ChatMessage::user(content)],
            format: None,
            tools: Vec::new(),
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

    /// Attaches tool definitions to the request.
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = tools;
        self
    }
}

/// Provider-agnostic completion response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Model that produced the response.
    pub model: String,
    /// Assistant text content.
    pub content: String,
    /// Whether the provider marked the response complete.
    pub done: bool,
    /// Tool calls requested by the assistant, if any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    /// Token and timing stats when the provider supplies them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics: Option<crate::metrics::CompletionMetrics>,
}

impl CompletionResponse {
    /// Returns true when the model requested tool invocations.
    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }
}
