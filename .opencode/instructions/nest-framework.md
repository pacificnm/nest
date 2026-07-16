# Nest framework usage

Nest is a Rust application framework in three layers: **core** (`core/crates/`),
**modules** (`modules/crates/`), and **apps** (separate git repositories, checked
out locally under `apps/<name>/` and gitignored here — see
[apps/README.md](../../apps/README.md)).

Before writing new infrastructure code — config loading, error types, an HTTP
client, a task scheduler, a data layer — check whether a Nest crate already
provides it:

- `search_project_memory` (`nest-memory` MCP) — prior architecture decisions and plans
- `search_knowledge_base` (`nest-knowledge` MCP) — indexed Rust/Tauri/React/Tailwind API docs
- The `nest-crates` skill — full crate catalog by purpose

If no suitable Nest crate exists, say so explicitly before introducing a new
external dependency or reimplementing common functionality from scratch.

When adapting an existing pattern from another Nest crate (e.g. how a module
loads its config section, how a client wraps errors), verify the exact idiom
against that crate's real source — don't guess at a plausible-looking variant.
API surfaces change; a remembered or assumed method name can be wrong.

## Read order for unfamiliar work

1. [docs/architecture.md](../../docs/architecture.md) — layering and dependency rules
2. [docs/app-standard.md](../../docs/app-standard.md) — product layout, hosts, IPC, command surface
3. `docs/plan/` — implementation plans
4. `docs/nest-<crate>/` and source under `core/crates/` and `modules/crates/`

See also: [dependency-rules.md](dependency-rules.md), [memory-workflow.md](memory-workflow.md).
