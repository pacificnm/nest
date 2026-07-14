# Nest MCP Workflow Skill

**Purpose**: Guide the opencode agent to use the three Nest MCP servers for memory, knowledge, and context management.

## MCP Servers Available

Three MCP servers are configured in `.cursor/mcp.json` and `opencode.json` (project root, mirrored in `.opencode/opencode.json`).

**Important:** Nest MCP servers expose **tools** only (e.g. `search_project_memory`), not MCP **resources**. Do not use `list_mcp_resources` to verify connectivity — use the Nest tool names directly or run `/mcp list` in OpenCode.

| Server | Tools | Purpose |
|--------|-------|---------|
| `nest-memory` | `search_project_memory` | Search indexed project docs, plans, architecture |
| `nest-knowledge` | `search_knowledge_base`, `list_knowledge_collections` | Search Rust/Tauri/React/Tailwind reference manuals |
| `nest-context-memory` | `save_context_memory`, `search_context_memory`, `list_context_memory`, `get_context_memory` | Persist agent session context across conversations |

## Mandatory Workflow (Always Follow)

### Before Implementation (Required)

**Step 1**: Search project memory
```
Tool: search_project_memory
Args: { query: "<task description>", limit: 8 }
```
Purpose: Find prior decisions, plans, and documentation about the task.

**Step 2**: Search or list context memory
```
Tool: search_context_memory OR list_context_memory
Args: { session_key: "<current-git-branch>", limit: 5 }
```
Purpose: Resume prior work from this session/branch.

**Step 3** (Optional, when using Rust/Tauri/React/Tailwind APIs): Search knowledge base
```
Tool: search_knowledge_base
Args: { query: "<API question>", collection: "rust-book" | "tauri" | "react" | "tailwind", limit: 8 }
```
Purpose: Get authoritative API documentation.

**Special case - Loon webOS client**: When editing `apps/loon/client/`, you MUST search knowledge with `collection="webos-tv"` (hook-enforced).

### During Work

- Use `nest-error` and `NestResult` for all error handling
- Respect layer boundaries (core → modules → apps)
- Prefer existing Nest crates over new implementations
- Follow conventions in `core/crates/nest-*` and `modules/crates/nest-*`

### After Every Response (Required)

**Save context memory** before the turn ends:
```
Tool: save_context_memory
Args: {
  content: "<what you did this turn>",
  title: "<short title>",
  session_key: "<git-branch-name>",
  tags: ["<relevant-tags>"]
}
```

Include:
- What you did this turn
- Files changed or read
- Decisions and blockers
- Verification commands and results

**Note**: Hooks will prompt you again if you skip this step.

### Before Context Compaction (Required)

Save a full checkpoint:
```
Tool: save_context_memory
Args: {
  content: "<full session summary>",
  title: "Session checkpoint",
  session_key: "<branch:conversation>"
}
```

## Knowledge Collections

Available collections in `nest-knowledge`:

- `rust-book` - The Rust Programming Language
- `rust-by-example` - Rust by Example
- `rust-reference` - Rust Reference
- `tauri` - Tauri documentation
- `react` - React documentation
- `tailwind` - Tailwind CSS documentation
- `webos-tv` - webOS TV development (for Loon client)

List collections:
```
Tool: list_knowledge_collections
```

## Read Order (Per AGENTS.md)

1. `docs/architecture.md` — layering and dependency rules
2. `docs/plan/` — implementation plans
3. MCP project memory (`search_project_memory`)
4. MCP knowledge base (`search_knowledge_base` for Rust/Tauri/React/Tailwind APIs)
5. MCP context memory (`search_context_memory` or `list_context_memory`)
6. Source code under `core/crates/` and `modules/crates/`

## Example Workflow

**User**: "Add a new endpoint to the Loon app that fetches data from an external API"

**Agent workflow**:

1. **Search project memory**:
   ```
   search_project_memory({ query: "Loon app HTTP client external API", limit: 8 })
   ```

2. **Search context memory**:
   ```
   list_context_memory({ session_key: "main", limit: 5 })
   ```

3. **Search knowledge** (for HTTP client API):
   ```
   search_knowledge_base({ query: "HTTP client GET request async", collection: "rust-book", limit: 8 })
   ```

4. **Implement** using `nest-http-client` crate

5. **Save context**:
   ```
   save_context_memory({
     content: "Added GET endpoint to Loon using nest-http-client. Files changed: apps/loon/src/api.rs. Verified with cargo check.",
     title: "Add HTTP endpoint to Loon",
     session_key: "main",
     tags: ["loon", "http", "api"]
   })
   ```

## Re-indexing

After documentation changes:
```bash
./scripts/index-memory.sh
```

After manual knowledge updates:
```bash
./scripts/index-knowledge.sh
```

## Setup Reference

See `tools/MCP-SETUP.md` for:
- PostgreSQL + pgvector setup
- Python venv and dependencies
- Indexing docs and knowledge
- Cursor configuration
- Hook enforcement

## Related Files

- `AGENTS.md` — Mandatory agent workflow
- `.cursor/mcp.json` — MCP server configuration
- `tools/MCP-SETUP.md` — Setup guide
- `docs/architecture.md` — Architecture and dependency rules
