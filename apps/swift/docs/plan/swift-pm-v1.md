# Swift PM v1

## Status: Planned

Phase **2** of [swift-v1](./swift-v1.md). Projects and tasks — backend commands + React views.

## Specs

- [projects](../specs/projects.md)
- [tasks](../specs/tasks.md)
- [ipc-api](../specs/ipc-api.md) — project + task commands
- [ui-platform](../specs/ui-platform.md)

## Prerequisites

- [swift-data-v1](./swift-data-v1.md) complete

## Rust (`src-tauri/`)

| Service | Role |
|---------|------|
| `ProjectService` | Active project, list, CRUD, archive |
| `TaskService` | CRUD, move, filter, labels |

Register services in `SwiftModule` or extend `SwiftDataModule`.

### Tauri commands

Implement project + task section of [ipc-api](../specs/ipc-api.md):

- `swift_list_projects`, `swift_create_project`, …
- `swift_list_tasks`, `swift_move_task`, …

Use `nest-tauri` command pattern from [`nest-tauri/src/commands.rs`](../../../core/crates/nest-tauri/src/commands.rs).

## React (`ui/`)

| Route | View |
|-------|------|
| `/` | Redirect to active project tasks |
| `/p/:slug/tasks` | List or kanban (toggle) |
| `/p/:slug/tasks/:id` | Task detail panel |

### Components

- `AppShell` — sidebar + main (see ui-platform spec)
- `ProjectSwitcher`
- `TaskList`, `KanbanBoard`, `TaskCard`, `TaskDetail`
- `TaskFilters` bar

### State

- React Query or lightweight fetch hooks calling `invoke`
- Optimistic update on kanban drag

## Phases

| Step | Deliverable |
|------|-------------|
| 2a | Project CRUD + switcher |
| 2b | Task list + create/edit |
| 2c | Kanban drag-and-drop |
| 2d | Labels + filters |

## Done when

- User can manage multiple projects and tasks entirely in UI
- Kanban drag persists status/order
- Manual QA checklist passes (create project → tasks → move columns)

## Related

- [swift-knowledge-v1](./swift-knowledge-v1.md)
- [swift-template-feedback-v1](./swift-template-feedback-v1.md)
