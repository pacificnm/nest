# Migrating Nest MCP to Remote Postgres + Ollama (192.168.88.10)

Status: **Executed.** `DATABASE_URL` was already pointing at `192.168.88.10` before
this migration (it was never on a local Unix socket in practice), so Phase 1
networking was already done and Phase 2/6 became an in-place re-embed on the one
existing database rather than an export/replay across two instances. The `postgres`
role was reused directly rather than creating a dedicated `nest_app` role. Phases
3-5 (schema resize to `vector(768)`, code changes, `.env`) ran as written below.
See `tools/MCP-SETUP.md` for the current setup instructions.

## What's changing

| Today | After migration |
| --- | --- |
| Postgres on `localhost`, Unix socket, peer auth | Postgres on `192.168.88.10:5432`, TCP, password auth |
| Embeddings: OpenAI `text-embedding-3-small` (1536-d) | Embeddings: Ollama `nomic-embed-text` on `192.168.88.10:11434` (768-d) |
| `OPENAI_API_KEY` required | No API key required for embeddings |

All three MCP servers (`nest-memory`, `nest-context-memory`, `nest-knowledge`) share
one Postgres database and one embedding pipeline, so this is a single coordinated
migration, not three separate ones.

**This forces a full re-index.** The embedding dimension is changing from 1536 to
768 — `pgvector` columns are fixed-width, and vectors from two different models
aren't comparable anyway. There's no in-place conversion. `project_memory` and
`knowledge_base` are cheap to rebuild (they're derived from source docs already in
the repo / on disk). `agent_context_memory` holds actual session notes that can't
be regenerated from source — Phase 2 below exports those before anything is
dropped.

## Files this touches

| File | Change |
| --- | --- |
| `tools/embedding.py` | Replace OpenAI call with an Ollama `/api/embed` call |
| `tools/index_memory.py` | Remove duplicated inline OpenAI call; use `embedding.embed_text` |
| `tools/search_memory.py` | Same |
| `tools/mcp_memory_server.py` | Same |
| `tools/context_memory.py` | Same |
| `tools/setup_database.sql` | `vector(1536)` → `vector(768)` on all three tables |
| `tools/setup_context_memory.sql` | Same, for `agent_context_memory` |
| `tools/requirements.txt` | Remove `openai` (no longer used anywhere) |
| `.env` | New `DATABASE_URL` (TCP), new `OLLAMA_HOST`, drop `OPENAI_API_KEY` |
| `.env.example` | Reflect the above for future clones |

`.cursor/mcp.json`, `.mcp.json`, and `opencode.json` **do not change** — they just
launch the local Python scripts by path. All the new config lives in `.env`, which
`tools/memory_common.py` already loads automatically.

---

## Phase 1 — Prepare the remote host (192.168.88.10)

### 1.1 Postgres + pgvector

If Postgres isn't already running there:

```bash
# on 192.168.88.10
sudo apt install postgresql postgresql-contrib postgresql-XX-pgvector   # match major version
sudo -u postgres createdb nest_memory
```

### 1.2 Allow remote (TCP) connections

Edit `postgresql.conf` (find it with `sudo -u postgres psql -c 'SHOW config_file;'`):

```conf
listen_addresses = 'localhost,192.168.88.10'
```

Edit `pg_hba.conf` to allow your dev machine's subnet with password auth:

```conf
# TYPE  DATABASE     USER   ADDRESS            METHOD
host    nest_memory  all    192.168.88.0/24    scram-sha-256
```

Restart Postgres:

```bash
sudo systemctl restart postgresql
```

Open the firewall port if one is active:

```bash
sudo ufw allow from 192.168.88.0/24 to any port 5432 proto tcp
```

### 1.3 Create a dedicated app role (password auth, since this is now TCP)

```bash
# on 192.168.88.10
sudo -u postgres psql -c "CREATE ROLE nest_app LOGIN PASSWORD '<choose-a-password>';"
```

Using a dedicated `nest_app` role instead of your OS username is recommended now
that the DB is reachable over the network — peer auth (`REPLACE_WITH_OS_USER`
matching `$USER`) doesn't apply over TCP.

### 1.4 Ollama

Already confirmed reachable at `192.168.88.10:11434`. Just make sure the
embedding model is pulled:

```bash
# on 192.168.88.10, or remotely with OLLAMA_HOST set
ollama pull nomic-embed-text
```

Verify from your dev machine:

```bash
curl http://192.168.88.10:11434/api/embed \
  -d '{"model": "nomic-embed-text", "input": "test"}'
```

You should get back JSON with a 768-length `embeddings[0]` array.

---

## Phase 2 — Preserve existing context-memory notes

`project_memory` and `knowledge_base` will be rebuilt from source in Phase 6 — skip
those. `agent_context_memory` holds real session notes (decisions, fixes, blockers)
that only exist in that table. Export them **before** touching schema or `.env`:

```bash
.venv/bin/python - <<'PY'
import json
import psycopg
from memory_common import database_url

with psycopg.connect(database_url()) as conn:
    rows = conn.execute(
        "SELECT session_key, title, content, tags, created_at "
        "FROM agent_context_memory ORDER BY id"
    ).fetchall()

entries = [
    {
        "session_key": r[0],
        "title": r[1],
        "content": r[2],
        "tags": r[3],
        "created_at": r[4].isoformat(),
    }
    for r in rows
]

with open("context_memory_export.json", "w") as f:
    json.dump(entries, f, indent=2)

print(f"Exported {len(entries)} entries to context_memory_export.json")
PY
```

Keep `context_memory_export.json` outside the repo (it's session content, not
something to commit) until Phase 6 replays it.

---

## Phase 3 — Update the schema for the new embedding dimension

In `tools/setup_database.sql` and `tools/setup_context_memory.sql`, change every
`embedding vector(1536)` to `embedding vector(768)` (three occurrences across the
two files: `project_memory`, `agent_context_memory`, `knowledge_base`).

Apply the updated schema on the remote host, over TCP, as the new `nest_app` role:

```bash
sed "s/REPLACE_WITH_OS_USER/nest_app/g" tools/setup_database.sql | \
  psql "postgresql://nest_app:<password>@192.168.88.10:5432/nest_memory" \
  -v ON_ERROR_STOP=1
```

(This is the same pattern `scripts/setup-database-postgres.sh` uses locally, just
piped to a remote TCP connection instead of a local `sudo -u postgres` socket.)

Verify:

```bash
psql "postgresql://nest_app:<password>@192.168.88.10:5432/nest_memory" \
  -c "\d project_memory"
```

Confirm `embedding` shows as `vector(768)`.

---

## Phase 4 — Code changes

### 4.1 `tools/embedding.py` — swap OpenAI for Ollama

```python
"""Shared chunking and embedding helpers for memory indexers."""

from __future__ import annotations

import os
from collections.abc import Iterator

import httpx

from memory_common import vector_literal

OLLAMA_HOST = os.environ.get("OLLAMA_HOST", "http://127.0.0.1:11434")
EMBEDDING_MODEL = os.environ.get("OLLAMA_EMBED_MODEL", "nomic-embed-text")
CHUNK_SIZE = 1800
CHUNK_OVERLAP = 200


def chunks(text: str, size: int = CHUNK_SIZE, overlap: int = CHUNK_OVERLAP) -> Iterator[str]:
    """Yield overlapping text chunks."""
    index = 0
    length = len(text)
    while index < length:
        yield text[index : index + size]
        index += size - overlap


def embed_text(text: str) -> str:
    """Return a pgvector literal for the embedded text."""
    response = httpx.post(
        f"{OLLAMA_HOST}/api/embed",
        json={"model": EMBEDDING_MODEL, "input": text},
        timeout=30.0,
    )
    response.raise_for_status()
    embedding = response.json()["embeddings"][0]
    return vector_literal(embedding)
```

`httpx` is already a dependency (used by the knowledge fetchers) — no new package
needed. Default `OLLAMA_HOST` stays `127.0.0.1` in code; the real address
(`192.168.88.10`) is set via `.env`, not hardcoded, so the source doesn't bake in
one machine's LAN IP.

### 4.2 Remove the duplicated inline embedding calls

Four other files each re-implement the same OpenAI call inline instead of using
`embedding.embed_text`. Point them at the shared helper instead:

- **`tools/index_memory.py`** — delete its local `embed()` function; call
  `from embedding import embed_text` and use `embed_text(chunk)` in place of
  `embed(chunk)`.
- **`tools/search_memory.py`** — replace the inline `OpenAI()` block with
  `from embedding import embed_text` and `embedding = embed_text(query)` (drop the
  `vector_literal(...)` wrapper call here since `embed_text` already returns the
  literal string).
- **`tools/mcp_memory_server.py`** — same change inside `search_project_memory`;
  replace the inline `OpenAI()` + `vector_literal(...)` block with
  `from embedding import embed_text` and `embedding = embed_text(query)`.
- **`tools/context_memory.py`** — delete its own `EMBEDDING_MODEL` constant and
  `embed_text()` function; `from embedding import embed_text` instead.

This isn't strictly required for the migration to work, but leaving four
independent copies of the embedding call means the next provider swap has to be
made four times instead of once. Worth doing in the same pass.

### 4.3 `tools/requirements.txt`

Remove the `openai` line — nothing in the codebase uses it once 4.1–4.2 are done
(confirmed: every reference to `openai`/`OpenAI` in `tools/` is one of the five
files above).

---

## Phase 5 — Update `.env`

```env
DATABASE_URL="postgresql://nest_app:<password>@192.168.88.10:5432/nest_memory"
OLLAMA_HOST="http://192.168.88.10:11434"
OLLAMA_EMBED_MODEL="nomic-embed-text"
NEST_KNOWLEDGE=/data/nest-knowledge
```

Drop `OPENAI_API_KEY` (no longer read by anything after Phase 4). Update
`.env.example` the same way so future clones don't get the stale OpenAI
instructions.

---

## Phase 6 — Re-index everything

With the new `.env` in place:

```bash
# Project docs → project_memory
./scripts/index-memory.sh

# Reference manuals → knowledge_base (sources already fetched locally)
./scripts/index-knowledge.sh --skip-fetch
```

Then replay the preserved context-memory notes from Phase 2, re-embedding each
through the new pipeline and keeping the original `created_at`:

```bash
.venv/bin/python - <<'PY'
import json
import psycopg
from embedding import embed_text
from memory_common import database_url

with open("context_memory_export.json") as f:
    entries = json.load(f)

with psycopg.connect(database_url()) as conn:
    for e in entries:
        embedding = embed_text(f"{e['title']}\n{e['content']}".strip())
        conn.execute(
            """
            INSERT INTO agent_context_memory
              (session_key, title, content, tags, embedding, created_at)
            VALUES (%s, %s, %s, %s, %s::vector, %s)
            """,
            (e["session_key"], e["title"], e["content"], e["tags"], embedding, e["created_at"]),
        )
    conn.commit()

print(f"Replayed {len(entries)} context memory entries.")
PY
```

---

## Phase 7 — Verify

```bash
.venv/bin/python tools/verify_memory_schema.py
.venv/bin/python tools/search_memory.py "nest-core module system"
.venv/bin/python tools/search_knowledge.py "invoke command" --collection tauri
.venv/bin/python tools/mcp_context_memory_server.py   # Ctrl+C after it starts cleanly
```

`verify_memory_schema.py` should report all three tables present with row counts
matching the re-index (and the replayed context-memory count).

Reload/restart whichever client is running the MCP servers (Cursor: **Developer:
Reload Window**; Claude Code / Kiwi / OpenCode: restart the agent session) so they
pick up the new `.env` values — the server processes themselves don't need any
config changes, since `.cursor/mcp.json` / `.mcp.json` / `opencode.json` only
reference script paths, not the database or embedding config.

---

## Rollback

Keep a copy of the old `.env` (`.env.local-backup`, untracked) with the local
`DATABASE_URL` and `OPENAI_API_KEY` until the remote setup is confirmed stable.
Don't drop the local `nest_memory` database or uninstall local Postgres during
this window — reverting is just restoring the old `.env` and reloading the MCP
clients, as long as the local DB still has its 1536-d data intact.

## Follow-up (not required to complete the migration)

- Update `tools/MCP-SETUP.md` prerequisites and troubleshooting table — it
  currently documents the OpenAI-key / local-socket setup as the only path.
- Consider whether `memory_common.py`'s `_PROVIDER_KEY_VARS` mapping for
  `OPENAI_API_KEY` is still needed once nothing reads that key.
