# nest-ai

AI inference provider contracts for the [Nest framework](../../README.md).

**Crate path:** [`core/crates/nest-ai`](../../core/crates/nest-ai)

## Role

nest-ai defines **what** inference means. Provider crates (`nest-ai-ollama`, future OpenAI/Gemini adapters) decide **how** HTTP calls happen.

Apps depend on [`AiProvider`](../../core/crates/nest-ai/src/provider.rs), not a specific engine:

```rust
use std::sync::Arc;
use nest_ai::{AiProvider, CompletionRequest};
use nest_ai_ollama::OllamaProvider;

let ai: Arc<dyn AiProvider> = Arc::new(OllamaProvider::new(config)?);
let response = ai
    .complete(CompletionRequest::user_message("Hello").with_json_format())
    .await?;
```

## Tool calling (Phase 2)

`CompletionRequest` accepts [`ToolDefinition`](../../core/crates/nest-ai/src/tools.rs) entries.
Responses and stream chunks may include [`ToolCall`](../../core/crates/nest-ai/src/tools.rs) values.
Use `ChatMessage::tool_result` for tool-role history entries.

Ollama adapter: [`nest-ai-ollama`](../nest-ai-ollama/README.md) sends `tools` on `/api/chat`
and parses `tool_calls` from responses and NDJSON stream lines.

## Providers

| Crate | Engine | Status |
|-------|--------|--------|
| [nest-ai-ollama](../nest-ai-ollama/README.md) | Ollama (`POST /api/chat`) | v0.1 |

## Related

- [Loon app](../../apps/loon/README.md) — filename guessing before TMDB search
