# Nest Memory MCP Setup

For the full setup guide, including Cursor configuration, see
[`MCP-SETUP.md`](MCP-SETUP.md).

This document describes the local setup for the Nest project-memory MCP server
in `tools/mcp_memory_server.py`.

The memory tools are development-time helpers only. They index repository
documentation into PostgreSQL with `pgvector`, then expose semantic search
through the MCP tool `search_project_memory`.

## Components

| File | Purpose |
| --- | --- |
| `tools/memory_common.py` | Shared environment loading, default database URL, pgvector literals. |
| `tools/setup_database.sql` | One-time DDL for all tables (`project_memory`, `agent_context_memory`, `knowledge_base`). |
| `tools/setup_database.py` | Python helper to create schema (non-superuser attempt first). |
| `scripts/index-memory.sh` | Shell wrapper that runs the indexer with the repo virtual environment. |
| `tools/index_memory.py` | Reads approved project context files and Markdown docs, creates embeddings. |
| `tools/search_memory.py` | Command-line semantic search against `project_memory`. |
| `tools/mcp_memory_server.py` | FastMCP stdio server exposing `search_project_memory(query, limit)`. |
| `tools/context_memory.py` | Save/search/list/get helpers for `agent_context_memory`. |
| `tools/mcp_knowledge_server.py` | FastMCP stdio server for reference manuals. |
| `tools/index_knowledge.py` | Index TOML-defined collections into knowledge_base. |
| `tools/search_knowledge.py` | CLI semantic search against knowledge_base. |
| `tools/memory_hooks.py` | Cursor `sessionStart` and `preCompact` hook handlers. |
| `.env.example` | Template for `DATABASE_URL` and `OPENAI_API_KEY`. |

## Database

Default connection:

```text
postgresql:///nest_memory?host=/var/run/postgresql
```

Tables:

| Table | Purpose |
| --- | --- |
| `project_memory` | Indexed repo Markdown (docs, plans, README). |
| `agent_context_memory` | Agent session checkpoints and compaction snapshots. |
| `knowledge_base` | External reference docs (indexer deferred). |

Embeddings use OpenAI `text-embedding-3-small` (1536 dimensions).

## Indexed paths

`tools/index_memory.py` indexes:

- `README.md`
- `AGENTS.md`
- `docs/**/*.md`
- `tools/MCP-SETUP.md`
- `tools/mcp-memory-setup.md`

Re-run `./scripts/index-memory.sh` after documentation changes.

## Agent usage

See [`AGENTS.md`](../AGENTS.md) for the recommended workflow:

1. `search_project_memory` before implementation
2. `search_context_memory` / `list_context_memory` when resuming
3. `save_context_memory` at checkpoints and before handoff

Read order: `docs/plan/` → MCP project memory → `docs/nest-*/` → source.

## Context memory

The context-memory MCP stores agent session notes in `agent_context_memory`.
Setup is included in `tools/setup_database.sql` — no separate migration needed.

Smoke test:

```bash
PYTHONPATH=tools .venv/bin/python - <<'PY'
from context_memory import save_context, search_context
entry_id = save_context("smoke test", title="setup check", session_key="setup")
print("saved", entry_id)
print("search hits", len(search_context("smoke test", session_key="setup")))
PY
```

## Knowledge base

The `knowledge_base` table is indexed via `./scripts/index-knowledge.sh` and
searched through MCP `nest-knowledge` (`search_knowledge_base`,
`list_knowledge_collections`).

Collections: `rust-book`, `rust-by-example`, `rust-reference`, `tauri`,
`react`, `tailwind`. Manuals live under `~/nest-knowledge` by default.

```bash
./scripts/index-knowledge.sh
.venv/bin/python tools/search_knowledge.py "useState" --collection react
```

## Troubleshooting

| Error | Fix |
| --- | --- |
| `Missing Python dependency for Nest memory MCP` | Run with `.venv/bin/python`; install `tools/requirements.txt`. |
| `type "vector" does not exist` | Install `pgvector` and run `setup_database.sql`. |
| `relation "project_memory" does not exist` | Run schema setup. |
| OpenAI authentication errors | Set `OPENAI_API_KEY` in `.env`. |
| Empty search results (project) | Run `./scripts/index-memory.sh`. |
| Empty search results (knowledge) | Run `./scripts/index-knowledge.sh`. |
