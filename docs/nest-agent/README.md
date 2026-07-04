# nest-agent

Tool-using agent loop for the [Nest framework](../../README.md).

**Crate path:** [`core/crates/nest-agent`](../../core/crates/nest-agent)

## Role

`nest-agent` orchestrates multi-step LLM completions with MCP tool execution:

1. Lists tools from [`nest_mcp::McpHub`](../nest-mcp/README.md)
2. Sends tool schemas to the model via [`nest_ai`](../nest-ai/README.md)
3. Executes approved tool calls and feeds results back until the model answers

## Quick start

```rust
use nest_agent::{AgentConfig, AgentLoop, CancelToken, ToolSource};
use nest_ai::{AiService, ChatMessage};
use nest_mcp::McpHub;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> nest_error::NestResult<()> {
    let ai: AiService = /* from OllamaModule */;
    let mut hub = McpHub::from_config_file(".cursor/mcp.json", None).await?;
    let (tx, mut rx) = mpsc::channel(32);

    let loop_ = AgentLoop::new(ai, AgentConfig::default());
    loop_
        .run(
            &mut hub,
            vec![ChatMessage::user("What is nest-core?")],
            None,
            tx,
            CancelToken::new(),
        )
        .await?;

    while let Some(event) = rx.recv().await {
        println!("{event:?}");
    }
    hub.shutdown().await?;
    Ok(())
}
```

## Related

- [Implementation plan](../plan/nest-agent-mcp-v1.md)
- [Kiwi plan](../../apps/kiwi/docs/agent-mcp-v1.md)
