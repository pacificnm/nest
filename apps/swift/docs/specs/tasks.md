# Swift — tasks

**Status:** Planned

## Scope

Task and subtask tracking within projects — list view, kanban board, labels, and filters.

## Requirements

### Task entity

| Field | Required | Notes |
|-------|----------|-------|
| `id` | yes | UUID |
| `project_id` | yes | FK to project |
| `parent_id` | no | Subtask parent |
| `title` | yes | |
| `description` | no | Markdown body |
| `status` | yes | Workflow column |
| `priority` | no | `none`, `low`, `medium`, `high`, `urgent` |
| `due_date` | no | Date only |
| `sort_order` | yes | Within status column / list |
| `created_at` / `updated_at` | yes | |
| `completed_at` | no | Set when entering done status |

### Status workflow (default)

| Status | Kanban column |
|--------|---------------|
| `backlog` | Backlog |
| `todo` | To Do |
| `in_progress` | In Progress |
| `review` | Review |
| `done` | Done |

Projects may override labels in v1.1; v1 uses workspace defaults.

### Labels

- Many-to-many tags on tasks (`labels` table + join)
- Filter by one or more labels
- Color optional per label

### Views

| View | Behavior |
|------|----------|
| **List** | Sortable columns; group by status optional |
| **Kanban** | Drag card between columns updates `status` + `sort_order` |
| **Detail** | Side panel or route: title, description, subtasks, linked notes |

### Filters

- By project (including “all active”)
- By status, priority, label
- Due: overdue, today, this week, none
- Text search on title + description

### Subtasks

- Shown nested under parent in detail view
- Subtask inherits `project_id`; `parent_id` set
- Completing all subtasks does not auto-complete parent (v1)

## Non-goals (v1)

- Dependencies between tasks
- Recurring tasks
- Gantt/timeline
- External issue import

## Related plans

- [swift-pm-v1](../plan/swift-pm-v1.md)
- [swift-data-v1](../plan/swift-data-v1.md)
