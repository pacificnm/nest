//! Streaming completion chunk types.

use std::pin::Pin;

use futures_core::Stream;

use crate::error::AiResult;
use crate::metrics::CompletionMetrics;
use crate::tools::ToolCall;

/// One incremental chunk from a streaming completion.
#[derive(Debug, Clone, PartialEq)]
pub struct CompletionChunk {
    /// Assistant text fragment to append.
    pub content_delta: String,
    /// Whether the provider marked the response complete.
    pub done: bool,
    /// Timing and token stats when the provider includes them on the final chunk.
    pub metrics: Option<CompletionMetrics>,
    /// Tool-call fragments emitted in this chunk.
    pub tool_calls: Vec<ToolCall>,
}

impl CompletionChunk {
    /// Creates a content delta chunk.
    pub fn delta(content: impl Into<String>) -> Self {
        Self {
            content_delta: content.into(),
            done: false,
            metrics: None,
            tool_calls: Vec::new(),
        }
    }

    /// Creates a terminal chunk with no further content.
    pub fn finished() -> Self {
        Self {
            content_delta: String::new(),
            done: true,
            metrics: None,
            tool_calls: Vec::new(),
        }
    }

    /// Creates a terminal chunk with provider metrics.
    pub fn finished_with_metrics(metrics: CompletionMetrics) -> Self {
        Self {
            content_delta: String::new(),
            done: true,
            metrics: Some(metrics),
            tool_calls: Vec::new(),
        }
    }
}

/// Type-erased streaming completion result.
pub type CompletionStream = Pin<Box<dyn Stream<Item = AiResult<CompletionChunk>> + Send>>;
