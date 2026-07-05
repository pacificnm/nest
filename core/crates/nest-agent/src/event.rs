//! Agent progress events for CLI and GUI hosts.

use std::time::Duration;

use nest_ai::CompletionMetrics;
use serde_json::Value;

/// Incremental agent loop events.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Assistant text fragment streamed from the model.
    TextDelta(String),
    /// A model step is starting.
    StepStarted {
        /// 1-based step index.
        step: u32,
    },
    /// Tool invocation started.
    ToolCallStarted {
        /// Model-visible tool name.
        tool: String,
        /// Arguments JSON.
        arguments: Value,
    },
    /// Tool returned successfully.
    ToolCallFinished {
        /// Model-visible tool name.
        tool: String,
        /// Tool output text.
        result: String,
        /// Wall time for the call.
        duration: Duration,
    },
    /// Tool failed or was rejected by policy.
    ToolCallFailed {
        /// Model-visible tool name.
        tool: String,
        /// Error message.
        error: String,
    },
    /// Agent finished with optional inference metrics from the last model call.
    Finished {
        /// Provider metrics when available.
        metrics: Option<CompletionMetrics>,
        /// Final assistant text.
        content: String,
    },
    /// Agent stopped with an error message.
    Failed(String),
}
