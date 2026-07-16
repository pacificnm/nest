# Tool: find-nest-crate

**Purpose**: Find the right Nest shared crate for a given task or feature need.

## When to invoke

- User asks "which crate should I use for..."
- User needs to find existing functionality in Nest crates
- User is implementing a feature and wants to reuse shared code
- User is unsure whether functionality exists in a Nest crate

## Usage

When the user describes a need, search the crate catalog and recommend the appropriate Nest crate(s).

## Crate Finder Guide

### If user needs...

| Need | Recommend |
|------|-----------|
| Error handling, error types | `nest-error` |
| Configuration loading | `nest-config` |
| Database access, repositories | `nest-data` |
| HTTP requests to external APIs | `nest-http-client` |
| Building a REST API server | `nest-http-serve` |
| Logging setup | `nest-logging` |
| Input/data validation | `nest-validation` |
| Caching data | `nest-cache` |
| Background tasks, async jobs | `nest-task`, `nest-task-runtime` |
| Desktop app with Tauri | `nest-tauri` |
| Terminal UI | `nest-tui` |
| CLI application | `nest-cli` |
| File I/O | `nest-file` |
| CSV files | `nest-file-csv` |
| Image processing | `nest-image`, `nest-media` |
| Theming, design tokens | `nest-theme`, `nest-design`, `nest-react-theme` |
| AI/LLM integration | `nest-ai` |
| Agent system | `nest-agent` |
| MCP (Model Context Protocol) | `nest-mcp` |
| Streaming data | `nest-stream` |
| Desktop UI, icons | `nest-tauri` + React `ui/` |
| App framework, modules | `nest-core`, `nest-app` |

## Search Strategy

1. **Understand the need**: What functionality does the user require?
2. **Match to crate**: Use the table above to identify candidate crates
3. **Verify existence**: Check `/data/projects/nest/core/crates/` or `/data/projects/nest/modules/crates/`
4. **Check dependencies**: Ensure the crate fits the user's layer (app vs module vs core)
5. **Provide guidance**: Show how to add and use the crate

## Example Interactions

**User**: "I need to make HTTP requests to an external API"
**Tool**: Recommend `nest-http-client` - provides HTTP client wrapper built on top of reqwest

**User**: "How do I handle errors in my Nest app?"
**Tool**: Recommend `nest-error` - provides `NestError` and `NestResult` types for consistent error handling

**User**: "I want to add caching to my app"
**Tool**: Recommend `nest-cache` - provides caching abstractions for performance optimization

## Related Tools

- `nest-knowledge_search_knowledge_base` - For API details and usage examples
- `nest-memory_search_project_memory` - For prior decisions about crate usage
- `glob` - To find crate source files
