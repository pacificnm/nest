# Nest agent workflow

**Mandatory** for all agents in this repository. Cursor hooks enforce the memory gate
and post-response saves.

## Before implementation (required — hook-enforced)

1. **`search_project_memory`** (`nest-memory`) — Nest crate plans, docs, decisions.
2. **`search_context_memory`** or **`list_context_memory`** (`nest-context-memory`)
   with `session_key` set to the current git branch (e.g. `main`).

Hooks **block** file edits, shell commands, and other tools until both complete.

Optional when using Rust/egui: **`search_knowledge_base`** (`nest-knowledge`).

## After every agent response (required)

Call **`save_context_memory`** before the turn ends. The **stop** hook will prompt
again if you skip it. Include:

- What you did this turn
- Files changed or read
- Decisions and blockers
- Verification commands and results

Use a stable `session_key` (git branch name or `branch:conversation`).

## Before context compaction (required)

Call **`save_context_memory`** with a full checkpoint. **preCompact** also saves an
automatic transcript snapshot, but you must still save an explicit summary.

## During work

- Prefer existing Nest conventions in `core/crates/nest-*`, `modules/crates/nest-*`, and `docs/nest-*`.
- Respect layer boundaries: **core** must not depend on modules or apps. See [docs/architecture.md](docs/architecture.md).
- Use `NestError` / `NestResult` from `nest-error`.

## Read order

1. [docs/architecture.md](docs/architecture.md) — layering and dependency rules
2. `docs/plan/` — implementation plans
3. MCP project memory
4. MCP knowledge base (Rust/egui APIs)
5. MCP context memory (resume prior work)
6. `docs/nest-<crate>/` and source under `core/crates/`, `modules/crates/`, `apps/crates/`

## Re-index

```bash
./scripts/index-memory.sh          # after doc changes
./scripts/index-knowledge.sh       # after manual updates
```

## Setup

See [`tools/MCP-SETUP.md`](tools/MCP-SETUP.md).
