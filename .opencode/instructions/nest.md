# Nest project instructions

Nest is a Rust application framework in three layers, each with its own
dependency direction:

| Layer | Path | May depend on | Must not depend on |
|-------|------|---------------|---------------------|
| **Core** | `core/crates/` | Other core crates | Modules, apps |
| **Modules** | `modules/crates/` | Core | Apps, other modules (avoid unless necessary) |
| **Apps** | separate repos, local checkout `apps/<name>/` | Core, modules (via `git` or `path` patch) | — |

No product source belongs in this monorepo — apps live in their own
repositories (`apps/README.md`).

Desktop apps follow one runtime model: business logic in Rust, `nest-tauri`
as the host, React as presentation only, IPC only at the webview boundary
(`ui/` calls `invoke(...)` → thin `#[tauri::command]` in `src-tauri/` →
delegates to `crates/core`). React does not replace Nest modules or
duplicate domain logic. Legacy egui crates (`nest-gui`, `nest-icon`) are not
for new work.

## Before writing new infrastructure code

Config loading, error types, an HTTP client, a task scheduler, a data layer,
Tauri command patterns, logging setup — check first, in order:

1. `nest-memory_search_project_memory` — indexes `README.md`, `AGENTS.md`,
   `docs/` (architecture, app-standard, per-crate READMEs including
   `nest-error`, `nest-logging`, `nest-tauri`), and app `docs/` dirs. This is
   the source of truth for Nest's own conventions and API idioms — use it
   instead of guessing at a remembered method name.
2. `nest-knowledge_search_knowledge_base` — indexed third-party reference
   manuals (currently `rust-book`, `rust-by-example`, `rust-reference`,
   `webos-tv`; check `nest-knowledge_list_knowledge_collections` for the
   current set before assuming `tauri`/`react`/`tailwind` collections exist).
3. The `nest-crates` skill — crate catalog by purpose.

If no suitable Nest crate exists, say so explicitly before adding an
external dependency or reimplementing something from scratch. When adapting
a pattern from another crate, verify the exact idiom via
`nest-memory_search_project_memory` against that crate's real docs — a
plausible-looking variant can be wrong.

## MCP tool naming

OpenCode namespaces every MCP tool as `<server>_<tool>`. Always use the full
prefixed name (e.g. `nest-memory_search_project_memory`) — the bare name
(`search_project_memory`) fails with "unavailable tool".

## Mandatory memory workflow

Before reading source, editing files, or running shell commands, both must
complete:

1. `nest-memory_search_project_memory` for plans, crate boundaries,
   architecture decisions, and prior work relevant to the task.
2. Determine the current git branch, call
   `nest-context-memory_list_context_memory` with it as `session_key`. If
   prior entries exist, `nest-context-memory_get_context_memory` the most
   relevant one; use `nest-context-memory_search_context_memory` when
   looking for a specific past decision rather than the most recent one.

During work, call `nest-context-memory_save_context_memory` after completing
a meaningful step, making an architectural decision, discovering a blocker,
running important verification, or changing approach.

**Before ending every response** — including read-only turns — call
`nest-context-memory_save_context_memory` (`session_key` = current git
branch) with: what you did, files read/changed, decisions, blockers,
verification run, and exact next steps. There is no hook enforcing this;
treat it as mandatory self-discipline. If the save fails, say so rather than
silently continuing.

OpenCode's `nest-context-memory` plugin auto-saves a checkpoint immediately
before compaction — do not also call
`nest-context-memory_save_context_memory` from the compaction summary agent.
