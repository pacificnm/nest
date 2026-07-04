# nest-mcp

MCP stdio client for the [Nest framework](../../README.md).

**Crate path:** [`core/crates/nest-mcp`](../../core/crates/nest-mcp)

## Role

`nest-mcp` spawns MCP server child processes (stdio, newline-delimited JSON), completes
the initialize handshake, and exposes [`McpHub`] for listing and calling tools.

Apps use this crate to connect Kiwi (and other hosts) to existing MCP servers such as
the Nest memory Python servers in [`tools/`](../../tools/).

## Quick start

```rust
use nest_mcp::McpHub;

#[tokio::main]
async fn main() -> nest_error::NestResult<()> {
    let mut hub = McpHub::from_config_file(".cursor/mcp.json", None).await?;
    let tools = hub.list_tools().await?;
    println!("tools: {:?}", tools.iter().map(|t| &t.qualified_name).collect::<Vec<_>>());
    hub.shutdown().await?;
    Ok(())
}
```

## Config

Loads Cursor-compatible [`.cursor/mcp.json`](../../.cursor/mcp.json):

```json
{
  "mcpServers": {
    "nest-memory": {
      "command": "/path/to/.venv/bin/python",
      "args": ["/path/to/tools/mcp_memory_server.py"],
      "cwd": "/path/to/nest"
    }
  }
}
```

## Related

- [Implementation plan](../plan/nest-agent-mcp-v1.md)
- [Kiwi plan](../../apps/kiwi/docs/agent-mcp-v1.md)
- [MCP-SETUP.md](../../tools/MCP-SETUP.md)
