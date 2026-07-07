# Swift — tasks

**Status:** Planned

## Scope

**Microsoft Project–style** task scheduling within projects. Primary surface is the **Gantt Chart** (entry table + timeline); alternate views match Project where practical.

## UX reference

Swift follows **Microsoft Project** interaction patterns:

| Project concept | Swift equivalent |
|-----------------|------------------|
| Gantt Chart | Default view — grid + timeline split |
| Task Sheet | Spreadsheet-only task table |
| Summary tasks | `is_summary` + outline indent |
| Predecessors | Dependency links (`FS` default) |
| % Complete | `percent_complete` |
| Task Information dialog | Task detail / properties panel |
| Indent / Outdent | Outline level on ribbon **Task** tab |
| Mark on Track | Schedule update (deferred) |

Kanban is **not** a primary view; optional board may come later for ad-hoc work.

## Task entity

| Field | Required | Notes |
|-------|----------|-------|
| `id` | yes | UUID |
| `project_id` | yes | FK to project |
| `parent_id` | no | Summary / outline parent |
| `outline_level` | yes | 0 = top level; indent depth |
| `is_summary` | yes | Roll-up row (bold in grid) |
| `is_milestone` | no | Zero-duration diamond on Gantt |
| `title` | yes | Task Name column |
| `notes` | no | Rich text / linked knowledge |
| `duration_minutes` | no | Working duration; milestone = 0 |
| `start_at` | no | Scheduled start (project calendar) |
| `finish_at` | no | Scheduled finish |
| `percent_complete` | yes | 0–100, default 0 |
| `priority` | no | Optional; not shown in v1 Gantt columns |
| `sort_order` | yes | Row order within outline |
| `created_at` / `updated_at` | yes | |
| `actual_start` / `actual_finish` | no | Tracking (phase 2b+) |

### Predecessors (dependencies)

Separate table `task_links`:

| Field | Notes |
|-------|-------|
| `predecessor_id` | Task that must complete/start first |
| `successor_id` | Dependent task |
| `link_type` | `FS` (default), `SS`, `FF`, `SF` |
| `lag_minutes` | Optional lag |

Displayed in **Predecessors** column as Project-style ids (e.g. `3FS+2d` deferred; v1 shows predecessor task row numbers).

### Resources (deferred)

Resource assignment sheet like Project is **v1.1**. v1 may show a single **Resource Names** text column for the solo user.

## Views (ribbon **View** tab)

| View | Route | Behavior |
|------|-------|----------|
| **Gantt Chart** | `/gantt` | Entry grid + timeline (default) |
| **Task Sheet** | `/task-sheet` | Full-width column grid |
| **Calendar** | `/calendar` | Month/week task bars |
| **Network Diagram** | `/network` | PERT chart (deferred after Gantt) |
| **Notes & Knowledge** | `/knowledge` | Swift extension — not in Project |

## Ribbon **Task** tab (v1 targets)

- New Task, Delete, Indent, Outdent
- Task Information
- Mark on Track, Respect Links (deferred)

## Filters

- By outline level / summary vs detail
- By date range, % complete, resource (later)
- Text search on task name

## Non-goals (v1)

- Full resource leveling and cost tables
- Baselines / earned value
- Recurring tasks
- Jira/GitHub issue import

## Related plans

- [swift-pm-v1](../plan/swift-pm-v1.md)
- [swift-data-v1](../plan/swift-data-v1.md)
- [ui-platform](ui-platform.md)
