# Swift PM v1

## Status: Planned

Phase **2** of [swift-v1](./swift-v1.md). **Microsoft Project–style** scheduling UI + backend.

## Specs

- [projects](../specs/projects.md)
- [tasks](../specs/tasks.md) — Gantt-first task model
- [ui-platform](../specs/ui-platform.md) — ribbon + Gantt shell
- [ipc-api](../specs/ipc-api.md)

## Prerequisites

- [swift-data-v1](./swift-data-v1.md) complete

## Rust (`src-tauri/`)

| Service | Role |
|---------|------|
| `ProjectService` | Portfolio CRUD, pin, archive, open |
| `TaskService` | CRUD, outline, predecessors, % complete |
| `ScheduleService` | Recalculate dates from links (v1 basic FS) |

### Tauri commands

Implement project + task sections of [ipc-api](../specs/ipc-api.md).

## React (`ui/`)

| Route | View |
|-------|------|
| `/gantt` | **Gantt Chart** (default) |
| `/task-sheet` | Task Sheet |
| `/calendar` | Calendar (phase 2c) |
| `/projects` | Project Center |
| `/settings` | Project Information |

### Components

- `ProjectRibbon`, `GanttChartView`, `TaskSheetView` — scaffolded
- `TaskGrid` — shared column model (extract from Gantt)
- `GanttTimeline` — real bars, drag, link lines (phase 2b+)
- `TaskInformationDialog` — modal form

### State

- `ProjectViewContext` — active view + route sync
- Fetch hooks / React Query for tasks per project

## Phases

| Step | Deliverable |
|------|-------------|
| 2a | Project Center + open project → Gantt |
| 2b | Task CRUD + entry grid bound to Postgres |
| 2c | Gantt timeline bars + predecessors (FS) |
| 2d | Calendar view, % complete, Mark on Track |

## Done when

- User opens a project and edits tasks in **Gantt Chart** and **Task Sheet**
- Indent/outdent creates summary outline
- Predecessor column persists `task_links`
- Portfolio of many projects via Project Center

## Related

- [swift-knowledge-v1](./swift-knowledge-v1.md)
- [swift-template-feedback-v1](./swift-template-feedback-v1.md)
