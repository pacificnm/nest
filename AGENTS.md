# Nest Codex workflow

**Mandatory** for Codex in this repository. Codex loads this file as repository
guidance and loads the three Nest MCP servers from `.codex/config.toml` in a trusted
checkout. Codex does not currently have the Cursor memory hooks, so treat the memory
gate and final-response save as hard workflow requirements.

## Before implementation (required)

Do not edit files, run shell commands, or use other non-memory tools until both calls
below succeed:

1. **`search_project_memory`** (`nest-memory`) — search Nest plans, documentation,
   crate boundaries, architecture decisions, and prior work relevant to the task.
2. **`search_context_memory`** or **`list_context_memory`** (`nest-context-memory`)
   with `session_key` set to the current git branch (for example, `main`) or a stable
   `branch:conversation` key.

If a required MCP server or tool is unavailable, stop before implementation and report
the configuration problem. Do not silently substitute local file search for the memory
calls.

Use **`search_knowledge_base`** (`nest-knowledge`) before relying on Rust, Tauri,
React, Tailwind, or other indexed API behavior.

When building **desktop UI** (`ui/`, Tauri): search collections `tauri`, `react`, `tailwind`.

When editing **`apps/loon/client/`**: **`search_knowledge_base`** with
`collection="webos-tv"` (hook-enforced). See `.cursor/rules/webos-tv-knowledge.mdc`.

## Before every final response (required)

Call **`save_context_memory`** (`nest-context-memory`) before sending the final response.
Save after read-only investigations as well as implementation turns. Include:

- What you did this turn
- Files changed or read
- Decisions and blockers
- Verification commands and results

Use the same stable `session_key` used at the start of the turn. If saving fails, say so
in the final response.

## Before context compaction (required)

Call **`save_context_memory`** with a full checkpoint before compaction. Codex has no
repository hook that guarantees this save, so do it explicitly.

## Nest Framework Usage

When implementing features inside `/apps`, always prefer using existing crates from `/core/crates`.

Before introducing a new dependency or implementing common functionality, check whether a Nest crate already provides the capability.

Examples:

- Data access → `nest-data`
- Validation → `nest-validate`
- Errors → `nest-error`
- Configuration → `nest-config`
- Logging → `nest-logging`
- HTTP APIs → `nest-http-serve`

If no suitable Nest crate exists, explain why before introducing a new dependency.

## During work

- Prefer existing Nest conventions in `core/crates/nest-*`, `modules/crates/nest-*`, and `docs/nest-*`.
- Respect layer boundaries: **core** must not depend on modules or apps. See [docs/architecture.md](docs/architecture.md).
- Use `NestError` / `NestResult` from `nest-error`.

## Read order

1. [docs/architecture.md](docs/architecture.md) — layering and dependency rules
2. [docs/app-standard.md](docs/app-standard.md) — product layout, hosts, IPC, command surface
3. `docs/plan/` — implementation plans
4. MCP project memory
5. MCP knowledge base (Rust, Tauri, React, Tailwind APIs)
6. MCP context memory (resume prior work)
7. `docs/nest-<crate>/` and source under `core/crates/` and `modules/crates/` (apps are separate repos — see `apps/README.md`)

## Re-index

```bash
./scripts/index-memory.sh          # after doc changes
./scripts/index-knowledge.sh       # after manual updates
```

## Setup

See [`tools/MCP-SETUP.md`](tools/MCP-SETUP.md).
