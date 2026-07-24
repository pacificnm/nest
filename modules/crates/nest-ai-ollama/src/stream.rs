//! Ollama NDJSON stream parsing.

use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;
use futures_util::StreamExt;
use nest_ai::{CompletionMetrics, ToolCall};
use nest_http_client::ByteStream;
use serde::Deserialize;

use crate::error::{OllamaError, OllamaResult};
use crate::message::{tool_calls_from_ollama, OllamaToolCall};

#[derive(Debug, Deserialize)]
struct StreamMessage {
    #[serde(default)]
    content: String,
    /// Extended thinking/reasoning text, present when the request set `think`
    /// on a model that supports it (e.g. qwen3, deepseek-r1). Streams before
    /// `content` — surfacing it is what keeps a "thinking" turn from looking
    /// frozen while the model reasons before producing (or instead of)
    /// visible content.
    #[serde(default)]
    thinking: String,
    #[serde(default)]
    tool_calls: Option<Vec<OllamaToolCall>>,
}

/// Incremental Ollama chat chunk.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatStreamChunk {
    /// Assistant text fragment.
    pub content: String,
    /// Extended thinking/reasoning text fragment, when `think` is enabled.
    pub thinking: String,
    /// Whether generation finished.
    pub done: bool,
    /// Token and timing stats on the final chunk.
    pub metrics: Option<CompletionMetrics>,
    /// Tool-call fragments from this chunk.
    pub tool_calls: Vec<ToolCall>,
}

/// Parses Ollama `POST /api/chat` NDJSON into discrete chunks.
pub struct ChatStream {
    bytes: ByteStream,
    buffer: String,
}

impl ChatStream {
    /// Wraps a raw HTTP byte stream.
    pub fn new(bytes: ByteStream) -> Self {
        Self {
            bytes,
            buffer: String::new(),
        }
    }
}

impl Stream for ChatStream {
    type Item = OllamaResult<ChatStreamChunk>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().get_mut();

        loop {
            if let Some(line) = next_line(&mut this.buffer) {
                return Poll::Ready(Some(parse_line(&line)));
            }

            match Pin::new(&mut this.bytes).poll_next(cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    this.buffer.push_str(&String::from_utf8_lossy(&chunk));
                }
                Poll::Ready(Some(Err(error))) => {
                    return Poll::Ready(Some(Err(OllamaError::from(error))));
                }
                Poll::Ready(None) => {
                    if this.buffer.trim().is_empty() {
                        return Poll::Ready(None);
                    }
                    let line = this.buffer.trim().to_string();
                    this.buffer.clear();
                    return Poll::Ready(Some(parse_line(&line)));
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

fn next_line(buffer: &mut String) -> Option<String> {
    let index = buffer.find('\n')?;
    let mut line = buffer.drain(..=index).collect::<String>();
    if line.ends_with('\n') {
        line.pop();
    }
    if line.ends_with('\r') {
        line.pop();
    }
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

#[derive(Debug, Deserialize)]
struct StreamLine {
    message: Option<StreamMessage>,
    done: bool,
    #[serde(default)]
    load_duration: u64,
    #[serde(default)]
    prompt_eval_count: u32,
    #[serde(default)]
    prompt_eval_duration: u64,
    #[serde(default)]
    eval_count: u32,
    #[serde(default)]
    eval_duration: u64,
    #[serde(default)]
    total_duration: u64,
}

fn parse_line(line: &str) -> OllamaResult<ChatStreamChunk> {
    let payload: StreamLine = serde_json::from_str(line).map_err(|error| {
        OllamaError::parse(format!("failed to parse ollama stream line: {error}"))
    })?;

    let (content, thinking, tool_calls) = payload
        .message
        .map(|message| {
            let calls = message
                .tool_calls
                .as_ref()
                .map(|calls| tool_calls_from_ollama(calls))
                .unwrap_or_default();
            (message.content, message.thinking, calls)
        })
        .unwrap_or_default();

    let metrics = payload.done.then(|| {
        CompletionMetrics::from_timing(
            payload.prompt_eval_count,
            payload.eval_count,
            payload.load_duration,
            payload.prompt_eval_duration,
            payload.eval_duration,
            payload.total_duration,
        )
    });

    Ok(ChatStreamChunk {
        content,
        thinking,
        done: payload.done,
        metrics,
        tool_calls,
    })
}

/// Collects a stream into a single assistant message.
pub async fn collect_chat_stream(mut stream: ChatStream) -> OllamaResult<String> {
    let mut content = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        content.push_str(&chunk.content);
        if chunk.done {
            break;
        }
    }
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_done_line_includes_metrics() {
        let line = r#"{
            "done": true,
            "prompt_eval_count": 37,
            "prompt_eval_duration": 643843000,
            "eval_count": 98,
            "eval_duration": 4538229000,
            "total_duration": 8908920680
        }"#;

        let chunk = parse_line(line).unwrap();
        assert!(chunk.done);
        let metrics = chunk.metrics.expect("metrics");
        assert_eq!(metrics.prompt_tokens, 37);
        assert_eq!(metrics.completion_tokens, 98);
        assert_eq!(metrics.total_tokens, 135);
    }

    #[test]
    fn parse_line_extracts_thinking_separately_from_content() {
        let line = r#"{
            "message": {
                "role": "assistant",
                "content": "",
                "thinking": "The user wants a trade setup, I should call calculate_trade_setup."
            },
            "done": false
        }"#;

        let chunk = parse_line(line).unwrap();
        assert_eq!(
            chunk.thinking,
            "The user wants a trade setup, I should call calculate_trade_setup."
        );
        assert_eq!(chunk.content, "");
    }

    #[test]
    fn parse_tool_calls_from_stream_line() {
        let line = r#"{
            "message": {
                "role": "assistant",
                "content": "",
                "tool_calls": [
                    {
                        "function": {
                            "name": "search_project_memory",
                            "arguments": {"query": "nest-core", "limit": 3}
                        }
                    }
                ]
            },
            "done": true
        }"#;

        let chunk = parse_line(line).unwrap();
        assert_eq!(chunk.tool_calls.len(), 1);
        assert_eq!(chunk.tool_calls[0].name, "search_project_memory");
        assert_eq!(
            chunk.tool_calls[0].arguments,
            json!({"query": "nest-core", "limit": 3})
        );
    }
}
