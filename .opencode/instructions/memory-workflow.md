# Mandatory memory workflow

Two separate MCP servers are involved. OpenCode exposes each server's tools
under a `<server>_<tool>` name — always call the full prefixed name below,
never the bare tool name, or the call will fail with "unavailable tool".

- `nest-memory` — project memory: plans, crate boundaries, architecture
  decisions, prior work. Provides `nest-memory_search_project_memory`.
- `nest-context-memory` — this session's own history, keyed by git branch.
  Provides `nest-context-memory_search_context_memory`,
  `nest-context-memory_list_context_memory`,
  `nest-context-memory_get_context_memory`,
  `nest-context-memory_save_context_memory`.

## At the beginning of work

Before reading source files, editing files, or running shell commands, both
of the following must complete:

1. **`nest-memory_search_project_memory`** — search for plans, crate
   boundaries, architecture decisions, and prior work relevant to the task.
2. Determine the current git branch, then call
   **`nest-context-memory_list_context_memory`** using it as `session_key`. If
   prior entries exist, call `nest-context-memory_get_context_memory` for the
   most relevant one. Use `nest-context-memory_search_context_memory` when
   looking for a specific prior decision rather than the most recent one.

Do not substitute filesystem searches or source reads for either step — a
memory-informed answer and a plausible-looking guess can both compile, but
only one is grounded in what actually happened before.

## During work

Call `nest-context-memory_save_context_memory` after:

- completing a meaningful implementation step;
- making an architectural decision;
- discovering a blocker;
- running important verification;
- changing the planned approach.

## Before ending a response

Call `nest-context-memory_save_context_memory` with:

- current task;
- work completed;
- files read or modified;
- decisions;
- blockers;
- commands and tests;
- exact next steps.

Use the current git branch as `session_key`.

## Compaction

OpenCode automatically creates an additional database checkpoint immediately
before compaction (see the `nest-context-memory` plugin's
`experimental.session.compacting` hook). Do not attempt to call
`nest-context-memory_save_context_memory` from the compaction summary agent —
that automatic checkpoint is the only save that happens at that point.

See also [nest-framework.md](nest-framework.md) and
[.cursor/rules/nest-memory.mdc](../../.cursor/rules/nest-memory.mdc) (the
equivalent Cursor-enforced workflow).
