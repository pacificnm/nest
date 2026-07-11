# Nest Memory Workflow (Mandatory)

This rule enforces the same memory workflow as `.cursor/rules/nest-memory.mdc`.

## Before Any Implementation Work

**REQUIRED** — hooks enforce this gate. Do not edit files, run shell commands, or use non-memory tools until both searches complete:

1. **`search_project_memory`** (`nest-memory` MCP) — Search for plans, crate boundaries, architecture decisions, and prior work in the Nest project memory.

2. **`search_context_memory`** or **`list_context_memory`** (`nest-context-memory` MCP) with `session_key` = current git branch (e.g., `main`) or `branch:conversation` — Retrieve context from the current session.

**Optional but recommended:**

3. **`search_knowledge_base`** (`nest-knowledge` MCP) when using Rust, Tauri, React, or Tailwind APIs.

4. **`search_knowledge_base`** with `collection="webos-tv"` when editing `apps/loon/client/` (hook-enforced).

## After Every Agent Response

**REQUIRED** — Call **`save_context_memory`** before the turn ends. The stop hook will prompt again if you skip this.

Include in your save:
- Summary of what changed this turn
- Files touched (read or written)
- Decisions made, blockers encountered, open questions
- Verification commands run and their results

Use a consistent `session_key` every time (git branch name or `branch:conversation`).

## Before Context Compaction

**REQUIRED** — Call **`save_context_memory`** with a full checkpoint summary. The `preCompact` hook also auto-snapshots the transcript, but an explicit agent save is still required.

## Nest Framework Guidelines

When implementing features inside `/apps`, always prefer using existing crates from `/core/crates` and `/modules/crates`.

Before introducing a new dependency or implementing common functionality, check whether a Nest crate already provides the capability:

- Data access → `nest-data`
- Validation → `nest-validate`
- Errors → `nest-error`
- Configuration → `nest-config`
- Logging → `nest-logging`
- HTTP APIs → `nest-http-serve`

If no suitable Nest crate exists, explain why before introducing a new dependency.

## Read Order

1. `docs/architecture.md` — layering and dependency rules
2. `docs/app-standard.md` — product layout, hosts, IPC, command surface
3. `docs/plan/` — implementation plans
4. MCP project memory (`search_project_memory`)
5. MCP knowledge base (`search_knowledge_base` for Rust/Tauri/React/Tailwind)
6. MCP context memory (`search_context_memory` to resume prior work)
7. `docs/nest-<crate>/` and source under `core/crates/` and `modules/crates/`

See [AGENTS.md](AGENTS.md) and [tools/MCP-SETUP.md](tools/MCP-SETUP.md) for setup details.
