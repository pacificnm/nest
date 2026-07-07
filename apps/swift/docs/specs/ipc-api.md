# Swift — IPC API

**Status:** Planned

## Scope

Tauri **commands** (request/response) and **events** (server push) between React `ui/` and Rust `src-tauri/`.

Follows [`nest-tauri`](../../../../docs/nest-tauri/README.md) patterns: structured types, `NestResult` errors serialized to webview.

## Built-in (nest-tauri)

Already provided when using `nest-tauri` with `runtime` (+ `images` optional):

| Command | Notes |
|---------|-------|
| `nest_app_metadata` | App name, window title |
| `nest_theme_css` | Active theme CSS variables |
| `nest_image_fetch` | Optional; remote images |

## Swift commands (v1)

### Projects

| Command | Request | Response |
|---------|---------|----------|
| `swift_list_projects` | `{ include_archived?: bool }` | `{ projects: Project[] }` |
| `swift_get_project` | `{ id: string }` | `Project` |
| `swift_create_project` | `CreateProjectRequest` | `Project` |
| `swift_update_project` | `UpdateProjectRequest` | `Project` |
| `swift_archive_project` | `{ id: string }` | `Project` |
| `swift_pin_project` | `{ id: string, pinned: bool }` | `Project` |
| `swift_set_focus_project` | `{ id: string }` | `{ ok: true }` — agent default + last route hint (stored in `app_settings`) |

### Tasks

| Command | Request | Response |
|---------|---------|----------|
| `swift_list_tasks` | `TaskFilter` | `{ tasks: Task[] }` |
| `swift_get_task` | `{ id: string }` | `Task` |
| `swift_create_task` | `CreateTaskRequest` | `Task` |
| `swift_update_task` | `UpdateTaskRequest` | `Task` |
| `swift_move_task` | `{ id, status, sort_order }` | `Task` |
| `swift_delete_task` | `{ id: string }` | `{ ok: true }` |

### Knowledge

| Command | Request | Response |
|---------|---------|----------|
| `swift_list_knowledge` | `{ project_id?, kind? }` | `{ items: KnowledgeItem[] }` |
| `swift_get_knowledge_item` | `{ id: string }` | `KnowledgeItem` |
| `swift_create_note` | `CreateNoteRequest` | `KnowledgeItem` (`kind=note`) |
| `swift_update_note` | `UpdateNoteRequest` | `KnowledgeItem` |
| `swift_import_doc` | `{ project_id, path, title? }` | `KnowledgeItem` (`kind=doc`) |
| `swift_search_knowledge` | `{ query, project_id?, kind?, mode?: vector\|keyword\|hybrid }` | `{ hits: SearchHit[] }` |
| `swift_delete_knowledge_item` | `{ id: string }` | `{ ok: true }` |

### Database

| Command | Request | Response |
|---------|---------|----------|
| `swift_db_health` | — | `{ ok: boolean, database: string }` |

### Agent

| Command | Request | Response |
|---------|---------|----------|
| `swift_agent_start` | `{ message, model?, context? }` | `{ run_id: string }` |
| `swift_agent_cancel` | `{ run_id: string }` | `{ ok: true }` |

### Settings

| Command | Request | Response |
|---------|---------|----------|
| `swift_get_settings` | — | `AppSettings` |
| `swift_update_settings` | `Partial<AppSettings>` | `AppSettings` |

## Events (agent stream)

| Event | Payload |
|-------|---------|
| `swift://agent/step-started` | `{ run_id, step }` |
| `swift://agent/text-delta` | `{ run_id, delta }` |
| `swift://agent/tool-started` | `{ run_id, tool, args_preview }` |
| `swift://agent/tool-finished` | `{ run_id, tool, result_preview, duration_ms }` |
| `swift://agent/finished` | `{ run_id, content }` |
| `swift://agent/failed` | `{ run_id, error }` |

UI subscribes via `@tauri-apps/api/event` for the active `run_id`.

## Error shape

Same as Nest IPC: `{ kind, message, code?, module? }` from `NestError` serde.

## Non-goals (v1)

- GraphQL or REST HTTP API
- Batch command multiplexing

## Related plans

- [swift-pm-v1](../plan/swift-pm-v1.md)
- [swift-agent-v1](../plan/swift-agent-v1.md)
