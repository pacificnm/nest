# Nest MCP Setup

This guide covers local setup for the Nest MCP servers and wiring them into
Cursor. The servers are development-time helpers only.

Three MCP servers are provided:

| Server | Script | Purpose |
| --- | --- | --- |
| `nest-memory` | `tools/mcp_memory_server.py` | Semantic search over indexed project docs and plans. |
| `nest-knowledge` | `tools/mcp_knowledge_server.py` | Semantic search over Rust, Tauri, React, Tailwind reference manuals. |
| `nest-context-memory` | `tools/mcp_context_memory_server.py` | Save, search, and retrieve agent session context across Cursor compaction. |

File tools (`read_file`, `write_file`, etc.) are **native Rust** in `nest-agent` via
`nest-file` — they are not an MCP server. Kiwi wires them in-process when running
the agent loop.

All servers use the same PostgreSQL database (`nest_memory`), Python virtual
environment, and OpenAI embedding key.

## Quick Start Checklist

From the repository root:

1. Install PostgreSQL and `pgvector`.
2. Create the `nest_memory` database and run `tools/setup_database.sql`.
3. Create `.venv`, install Python dependencies, and create `.env`.
4. Index project docs: `./scripts/index-memory.sh`
5. Verify all tables: `.venv/bin/python tools/verify_memory_schema.py`
6. Index reference manuals: `./scripts/index-knowledge.sh` (auto-fetches sources into `/data/nest-knowledge` or `$NEST_KNOWLEDGE`)

## Prerequisites

- PostgreSQL with the `pgvector` extension
- Python 3
- Git (for `./scripts/fetch-knowledge.sh`)
- An OpenAI API key
- Cursor with MCP support

Default database URL:

```text
postgresql:///nest_memory?host=/var/run/postgresql
```

Override with `DATABASE_URL` in `.env` when using TCP, a remote host, or different
credentials.

## 1. PostgreSQL Setup

On Debian-family systems:

```bash
sudo apt install postgresql postgresql-contrib postgresql-XX-pgvector
```

Replace `XX` with the installed PostgreSQL major version.

Create the database:

```bash
sudo -u postgres createdb nest_memory
```

Enable `pgvector` and create all memory tables.

**Recommended** (pipes SQL via stdin — works when `psql -f` gets Permission denied
because postgres cannot read your home directory):

```bash
./scripts/setup-database-postgres.sh
```

Equivalent manual command:

```bash
sed "s/REPLACE_WITH_OS_USER/$USER/g" tools/setup_database.sql | sudo -u postgres psql nest_memory
```

Do **not** use `sudo -u postgres psql ... -f /home/.../setup_database.sql` unless the
SQL file is readable by the `postgres` OS user (often fails under `~/`).

`./scripts/setup-memory.sh` tries the Python helper first, then runs
`setup-database-postgres.sh` automatically if your user lacks CREATE on `public`.

Grant access to the OS user that will run the MCP servers (included in
`setup-database-postgres.sh`; run manually only if needed):

```bash
sudo -u postgres createuser "$USER" 2>/dev/null || true
sudo -u postgres psql -d nest_memory -c "GRANT ALL PRIVILEGES ON DATABASE nest_memory TO \"$USER\";"
sudo -u postgres psql -d nest_memory -c "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO \"$USER\";"
sudo -u postgres psql -d nest_memory -c "GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO \"$USER\";"
```

## 2. Python Setup

From the repository root:

```bash
python3 -m venv .venv
. .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install -r tools/requirements.txt
cp .env.example .env
```

Edit `.env`:

```env
DATABASE_URL="postgresql:///nest_memory?host=/var/run/postgresql"
OPENAI_API_KEY="sk-..."
```

Do not commit `.env`.

The MCP servers load `.env` automatically from the repository root through
`tools/memory_common.py`. You do not need to duplicate secrets in the Cursor
config.

Alternatively, run the setup helper:

```bash
./scripts/setup-memory.sh
```

## 3. Index Project Memory

Build the searchable doc index:

```bash
./scripts/index-memory.sh
```

The indexer reads `README.md`, `AGENTS.md`, all Markdown under `docs/`, and
the MCP setup docs under `tools/`. Re-run this after significant documentation
changes.

Verify search before wiring Cursor:

```bash
.venv/bin/python tools/search_memory.py "nest-core module system"
.venv/bin/python tools/search_knowledge.py "invoke command" --collection tauri
```

## 3b. Index Knowledge Base

Reference manuals (Rust book, Tauri, React, Tailwind, webOS TV) live outside the repo under
`/data/nest-knowledge` or `$NEST_KNOWLEDGE`. Fetch and index them:

```bash
./scripts/index-knowledge.sh
```

This runs `./scripts/fetch-knowledge.sh` first (git clones + webOS doc fetch), then
indexes all collections. To skip re-fetching when sources are already present:

```bash
./scripts/index-knowledge.sh --skip-fetch
```

Fetch sources only (no indexing):

```bash
./scripts/fetch-knowledge.sh
./scripts/fetch-knowledge.sh --git-only    # Rust + Tauri/React/Tailwind only
./scripts/fetch-knowledge.sh --webos-only  # webOS TV docs only
./scripts/fetch-knowledge.sh --force       # git pull + re-fetch webOS pages
```

Override the root:

```bash
NEST_KNOWLEDGE=/path/to/manuals ./scripts/index-knowledge.sh
```

This writes `tools/knowledge.toml` and indexes collections: `rust-book`,
`rust-by-example`, `rust-reference`, `tauri`, `react`, `tailwind`.

List indexed collections:

```bash
.venv/bin/python tools/search_knowledge.py --list-collections
```

Re-run after updating manual checkouts. Indexing uses OpenAI embeddings and may
take several minutes.

## 3c. Context memory table

The `nest-context-memory` MCP server uses table **`agent_context_memory`**. It is
created by the same schema setup as the other memory tables — not a separate
database.

If you indexed project docs but never ran full schema setup, you may only have
`project_memory`. Fix:

```bash
./scripts/setup-context-memory.sh
```

Or create all tables at once:

```bash
./scripts/setup-database-postgres.sh
```

Verify:

```bash
.venv/bin/python tools/verify_memory_schema.py
```

You should see `agent_context_memory (ok)`. `./scripts/setup-memory.sh` runs this
check and a context save/search smoke test automatically.

## 4. Configure Cursor

Cursor reads MCP server definitions from JSON. This repository ships a
project-local config at `.cursor/mcp.json`.

Open `.cursor/mcp.json` and confirm the paths match your checkout:

```json
{
  "mcpServers": {
    "nest-memory": {
      "command": "/absolute/path/to/nest/.venv/bin/python",
      "args": ["/absolute/path/to/nest/tools/mcp_memory_server.py"],
      "cwd": "/absolute/path/to/nest"
    },
    "nest-context-memory": {
      "command": "/absolute/path/to/nest/.venv/bin/python",
      "args": ["/absolute/path/to/nest/tools/mcp_context_memory_server.py"],
      "cwd": "/absolute/path/to/nest"
    },
    "nest-knowledge": {
      "command": "/absolute/path/to/nest/.venv/bin/python",
      "args": ["/absolute/path/to/nest/tools/mcp_knowledge_server.py"],
      "cwd": "/absolute/path/to/nest"
    }
  }
}
```

Replace `/absolute/path/to/nest` with your local clone path (`pwd` from repo root).

Notes:

- Use absolute paths for `command`, `args`, and `cwd`.
- Keep secrets out of `mcp.json`. The servers read `DATABASE_URL` and
  `OPENAI_API_KEY` from repo-root `.env`.
- The committed file may contain another developer's path. Update it locally
  after clone.

### Reload Cursor

1. Save `.cursor/mcp.json`.
2. Open the command palette: **Ctrl+Shift+P**.
3. Run **Developer: Reload Window**.

### Verify in Cursor

1. Open **Settings**.
2. Go to **Tools & MCP**.
3. Confirm all servers appear and are connected:
   - `nest-memory` — 1 tool
   - `nest-knowledge` — 2 tools
   - `nest-context-memory` — 4 tools

### Manual server smoke test

```bash
.venv/bin/python tools/mcp_memory_server.py
.venv/bin/python tools/mcp_knowledge_server.py
.venv/bin/python tools/mcp_context_memory_server.py
```

Each process waits on stdio. Stop with **Ctrl+C**.

## 5. Cursor Hooks (memory enforcement — mandatory)

Project hooks in `.cursor/hooks.json` enforce the agent memory workflow:

| Hook | Script | Behavior |
| --- | --- | --- |
| `sessionStart` | `.cursor/hooks/memory_session_start.sh` | Resets gate; injects mandatory memory instructions + recent context. |
| `preToolUse` | `.cursor/hooks/memory_pre_tool_use.sh` | **Blocks** non-memory tools until `search_project_memory` and context read complete. |
| `postToolUse` | `.cursor/hooks/memory_post_tool_use.sh` | Records successful memory MCP calls to open the gate. |
| `afterMCPExecution` | `.cursor/hooks/memory_after_mcp.sh` | Same tracking for MCP tool completion. |
| `preCompact` | `.cursor/hooks/memory_pre_compact.sh` | Auto-snapshots transcript; reminds agent to `save_context_memory`. |
| `stop` | `.cursor/hooks/memory_stop.sh` | **Follow-up** if agent did not `save_context_memory` this turn (`loop_limit: 2`). |

Gate state is stored in `.cursor/memory-gate-state.json` (gitignored).

Persistent rule: [`.cursor/rules/nest-memory.mdc`](../.cursor/rules/nest-memory.mdc) (`alwaysApply: true`).

Smoke tests:

```bash
echo '{"conversation_id":"test"}' | .cursor/hooks/memory_session_start.sh | jq .
echo '{"tool_name":"Shell"}' | .cursor/hooks/memory_pre_tool_use.sh | jq .
echo '{"conversation_id":"test"}' | .cursor/hooks/memory_stop.sh | jq .
```

## 6. MCP Tools

### Project memory (`nest-memory`)

| Tool | Arguments | Result |
| --- | --- | --- |
| `search_project_memory` | `query: str`, `limit: int = 8` | Matching doc snippets grouped by source path. |

### Knowledge base (`nest-knowledge`)

| Tool | Arguments | Result |
| --- | --- | --- |
| `search_knowledge_base` | `query: str`, `limit: int = 8`, `collection: str = ""` | Matching manual snippets (Rust, Tauri, React, Tailwind, …). |
| `list_knowledge_collections` | — | Indexed collection names. |

Pass `collection` to narrow (e.g. `"tauri"`, `"react"`, `"tailwind"`, `"rust-book"`, `"webos-tv"`).

**Loon webOS client:** hooks require `search_knowledge_base` with `collection="webos-tv"`
before editing `apps/loon/client/`. Index with `./scripts/fetch-webos-knowledge.sh`.
See `.cursor/rules/webos-tv-knowledge.mdc`.

### Context memory (`nest-context-memory`)

| Tool | Arguments | Result |
| --- | --- | --- |
| `save_context_memory` | `content`, `title`, `session_key`, `tags` | Stores one context entry and returns its id. |
| `search_context_memory` | `query`, `limit`, `session_key` | Semantic search over saved context. |
| `list_context_memory` | `limit`, `session_key` | Recent saved entries, newest first. |
| `get_context_memory` | `entry_id` | Full content for one entry. |

Use a stable `session_key` such as a branch name or task slug so related entries
stay grouped across compaction.

## 7. Using MCP from Kiwi

Kiwi Agent mode (chat panel **Agent** checkbox) runs the same MCP servers as Cursor
via `[agent]` in `apps/kiwi/desktop/config.toml`:

```toml
[agent]
model = "qwen2.5:7b"
mcp_config = "../../../.cursor/mcp.json"
mcp_servers = ["nest-memory", "nest-knowledge", "nest-context-memory"]
disabled_mcp_servers = []          # toggle off in Agent sidebar
allow_save_context = false         # opt-in for save_context_memory
allow_file_writes = false          # opt-in for write/update/delete/mkdir file tools
agent_mode = true
max_steps = 10
```

- **Agent sidebar** — enable/disable individual MCP servers; optional
  `save_context_memory` auto-run.
- **Tool Activity** tab — MCP call log with expandable results.
- **Streaming** — final agent replies stream token-by-token when Ollama supports it.
- **Read-only by default** — search/list/get/read tools auto-run; writes require
  `allow_save_context = true` or `allow_file_writes = true`.

See [`apps/kiwi/docs/agent-mcp-v1.md`](../apps/kiwi/docs/agent-mcp-v1.md).

## 8. Using MCP from OpenCode (Kiwi Agent panel)

When Kiwi launches **OpenCode** via `ollama launch opencode`, MCP is configured separately
from the Tools sidebar probe. OpenCode reads the `mcp` block in the workspace
[`opencode.json`](../opencode.json) (mirrored in [`.opencode/opencode.json`](../.opencode/opencode.json)).

Kiwi sets `OPENCODE_CONFIG` and `NEST_PROJECT_ROOT` when the Agent panel starts OpenCode.
The MCP `command` paths in `opencode.json` use `{env:NEST_PROJECT_ROOT}` so servers work even
when the Kiwi workspace is a nested folder (e.g. `apps/kiwi`).

**Nest MCP provides tools, not resources.** OpenCode may call `list_mcp_resources` and get an
empty list even when servers are connected. Use `search_project_memory`, `search_knowledge_base`,
or `/mcp list` instead.

Verify from a terminal in the repo root:

```bash
ollama launch opencode --model qwen3.5:397b-cloud
# inside OpenCode:
/mcp list
```

Requirements (same as Cursor):

- `.venv/bin/python` with `tools/requirements.txt` installed
- Repo `.env` with `DATABASE_URL` and `OPENAI_API_KEY` (for embeddings)
- Indexed project memory (`./scripts/index-memory.sh`)

The [Ollama tool-calling docs](https://docs.ollama.com/capabilities/tool-calling) describe
the inference API; OpenCode handles the tool loop and MCP stdio servers itself.

## Troubleshooting

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| MCP server missing after reload | Wrong config file or invalid JSON | Confirm `.cursor/mcp.json` exists and parses as JSON. |
| MCP server fails immediately | Bad Python path or missing venv | Update `command` to your local `.venv/bin/python`. |
| `Missing Python dependency for Nest memory MCP` | Dependencies not installed | Run `pip install -r tools/requirements.txt` inside `.venv`. |
| OpenAI authentication errors | Missing or invalid API key | Set `OPENAI_API_KEY` in `.env`. |
| `relation "agent_context_memory" does not exist` | Context table not created | Run `./scripts/setup-context-memory.sh` or `./scripts/setup-database-postgres.sh`. |
| `relation "project_memory" does not exist` | Schema not created | Run `./scripts/setup-database-postgres.sh`. |
| `Permission denied` on `psql -f ~/.../setup_database.sql` | postgres cannot read your home dir | Pipe SQL: `sed ... \| sudo -u postgres psql nest_memory` or use `./scripts/setup-database-postgres.sh`. |
| Empty project-memory search results | Index not built | Run `./scripts/index-memory.sh`. |
| Empty knowledge search results | Manuals not indexed | Run `./scripts/index-knowledge.sh` (fetches sources automatically). |
| PostgreSQL connection errors | Service down or role mismatch | Confirm PostgreSQL is running and `DATABASE_URL` matches your setup. |

## Related Files

| File | Purpose |
| --- | --- |
| `.cursor/mcp.json` | Cursor MCP client configuration. |
| `.cursor/hooks.json` | Cursor hook definitions. |
| `.env.example` | Template for local secrets and database URL. |
| `AGENTS.md` | Agent workflow and memory usage rules. |
| `tools/setup_database.sql` | One-time DDL for all memory tables. |
| `scripts/index-memory.sh` | Index project documentation. |
| `scripts/index-knowledge.sh` | Index Rust / Tauri / React / Tailwind reference manuals. |
| `tools/index_knowledge.py` | Knowledge indexer (TOML collections). |
| `tools/search_knowledge.py` | CLI search against knowledge_base. |
| `tools/mcp_knowledge_server.py` | MCP server for reference manuals. |
| `scripts/setup-memory.sh` | Python schema setup helper. |
| `tools/mcp-memory-setup.md` | Additional reference for memory internals. |
