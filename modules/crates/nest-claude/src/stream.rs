//! Server-sent-event types and parsing for streaming `POST /v1/messages`.

use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::{Stream, StreamExt};
use nest_http_client::ByteStream;
use serde::Deserialize;

use crate::error::{ClaudeError, ClaudeResult};
use crate::response::MessageResponse;
use crate::types::{StopReason, Usage};

/// Incremental content for a `content_block_delta` event.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentDelta {
    /// Incremental text for a `text` content block.
    TextDelta {
        /// The text chunk.
        text: String,
    },
    /// Incremental JSON for a `tool_use` block's `input`.
    InputJsonDelta {
        /// A partial JSON fragment; concatenate across deltas, then parse once complete.
        partial_json: String,
    },
    /// Incremental reasoning text for a `thinking` block.
    ThinkingDelta {
        /// The reasoning text chunk.
        thinking: String,
    },
    /// The final signature for a `thinking` block.
    SignatureDelta {
        /// Opaque signature; preserve verbatim when replaying the block.
        signature: String,
    },
}

/// Message-level fields carried on a `message_delta` event.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageDeltaFields {
    /// Why generation stopped, once known.
    pub stop_reason: Option<StopReason>,
    /// The custom stop sequence hit, if any.
    pub stop_sequence: Option<String>,
}

/// Body of an `error` stream event.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorBody {
    /// The Anthropic error type, e.g. `overloaded_error`.
    #[serde(rename = "type")]
    pub error_type: String,
    /// A human-readable error message.
    pub message: String,
}

/// A single Server-Sent Event from the streaming Messages API.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// The response has started; carries an initial (empty-content) message.
    MessageStart {
        /// The initial message shell.
        message: MessageResponse,
    },
    /// A new content block has started at `index`.
    ContentBlockStart {
        /// The content block's index in `message.content`.
        index: usize,
        /// The starting (usually empty) content block.
        content_block: serde_json::Value,
    },
    /// Incremental content for the block at `index`.
    ContentBlockDelta {
        /// The content block's index in `message.content`.
        index: usize,
        /// The incremental delta.
        delta: ContentDelta,
    },
    /// The content block at `index` is complete.
    ContentBlockStop {
        /// The content block's index in `message.content`.
        index: usize,
    },
    /// Message-level fields (`stop_reason`) and incremental usage.
    MessageDelta {
        /// Updated message-level fields.
        delta: MessageDeltaFields,
        /// Usage accumulated so far.
        usage: Usage,
    },
    /// The response is complete.
    MessageStop,
    /// A keep-alive heartbeat; carries no data.
    Ping,
    /// The API reported an error mid-stream.
    Error {
        /// The error body.
        error: ApiErrorBody,
    },
}

/// A stream of [`StreamEvent`]s from a streaming Messages API request.
pub struct MessageStream {
    inner: Pin<Box<dyn Stream<Item = ClaudeResult<StreamEvent>> + Send>>,
}

impl MessageStream {
    pub(crate) fn new(bytes: ByteStream) -> Self {
        let state = SseState {
            bytes,
            buffer: Vec::new(),
            pending: VecDeque::new(),
            done: false,
        };
        let inner = futures_util::stream::unfold(state, next_event).boxed();
        Self { inner }
    }
}

impl Stream for MessageStream {
    type Item = ClaudeResult<StreamEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().inner.as_mut().poll_next(cx)
    }
}

struct SseState {
    bytes: ByteStream,
    buffer: Vec<u8>,
    pending: VecDeque<ClaudeResult<StreamEvent>>,
    done: bool,
}

async fn next_event(mut state: SseState) -> Option<(ClaudeResult<StreamEvent>, SseState)> {
    loop {
        if let Some(event) = state.pending.pop_front() {
            return Some((event, state));
        }

        if state.done {
            return None;
        }

        match state.bytes.next().await {
            Some(Ok(chunk)) => {
                state.buffer.extend_from_slice(&chunk);
                drain_events(&mut state.buffer, &mut state.pending);
            }
            Some(Err(error)) => {
                state.done = true;
                return Some((Err(ClaudeError::from(error)), state));
            }
            None => {
                state.done = true;
                if !state.buffer.is_empty() {
                    let mut buffer = std::mem::take(&mut state.buffer);
                    buffer.extend_from_slice(b"\n\n");
                    drain_events(&mut buffer, &mut state.pending);
                }
            }
        }
    }
}

/// Extracts every complete `\n\n`-delimited SSE event from `buffer`, appending
/// parsed [`StreamEvent`]s (skipping `ping` events) to `pending`.
fn drain_events(buffer: &mut Vec<u8>, pending: &mut VecDeque<ClaudeResult<StreamEvent>>) {
    const DELIMITER: &[u8] = b"\n\n";

    while let Some(position) = find_subslice(buffer, DELIMITER) {
        let event_bytes: Vec<u8> = buffer.drain(..position + DELIMITER.len()).collect();
        let event_bytes = &event_bytes[..event_bytes.len() - DELIMITER.len()];

        if let Some(data) = extract_data(event_bytes) {
            if data.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            match serde_json::from_slice::<StreamEvent>(&data) {
                Ok(StreamEvent::Ping) => {}
                Ok(event) => pending.push_back(Ok(event)),
                Err(error) => pending.push_back(Err(ClaudeError::parse(format!(
                    "failed to decode SSE event: {error}"
                )))),
            }
        }
    }
}

/// Joins every `data:` line within one SSE event block into a single payload.
fn extract_data(event_bytes: &[u8]) -> Option<Vec<u8>> {
    const PREFIX: &[u8] = b"data:";
    let mut data = Vec::new();
    let mut found = false;

    for line in event_bytes.split(|byte| *byte == b'\n') {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if let Some(rest) = line.strip_prefix(PREFIX) {
            found = true;
            let rest = rest.strip_prefix(b" ").unwrap_or(rest);
            if !data.is_empty() {
                data.push(b'\n');
            }
            data.extend_from_slice(rest);
        }
    }

    found.then_some(data)
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream;
    use nest_error::NestResult;

    fn byte_stream(chunks: Vec<&'static str>) -> ByteStream {
        let items: Vec<NestResult<bytes::Bytes>> = chunks
            .into_iter()
            .map(|chunk| Ok(bytes::Bytes::from_static(chunk.as_bytes())))
            .collect();
        Box::pin(stream::iter(items))
    }

    #[tokio::test]
    async fn parses_text_delta_events() {
        let sse = "event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hi\"}}\n\n";
        let mut stream = MessageStream::new(byte_stream(vec![sse]));
        let event = stream.next().await.unwrap().unwrap();
        match event {
            StreamEvent::ContentBlockDelta { index, delta } => {
                assert_eq!(index, 0);
                match delta {
                    ContentDelta::TextDelta { text } => assert_eq!(text, "Hi"),
                    other => panic!("unexpected delta: {other:?}"),
                }
            }
            other => panic!("unexpected event: {other:?}"),
        }
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn skips_ping_events() {
        let sse = "event: ping\ndata: {\"type\":\"ping\"}\n\n\
event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n";
        let mut stream = MessageStream::new(byte_stream(vec![sse]));
        let event = stream.next().await.unwrap().unwrap();
        assert!(matches!(event, StreamEvent::MessageStop));
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn handles_events_split_across_chunks() {
        let chunk_a = "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",";
        let chunk_b = "\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hi\"}}\n\n";
        let mut stream = MessageStream::new(byte_stream(vec![chunk_a, chunk_b]));
        let event = stream.next().await.unwrap().unwrap();
        assert!(matches!(event, StreamEvent::ContentBlockDelta { .. }));
    }
}
