# Swift — personal tracking

**Status:** Planned

## Scope

Optional **activity log** and **time-on-task** for personal accountability — not billing or team reporting.

## Requirements

### Activity events

Append-only log of user actions:

| Event type | Payload |
|------------|---------|
| `task_created` | `task_id`, `project_id` |
| `task_status_changed` | `task_id`, `from`, `to` |
| `task_completed` | `task_id` |
| `note_created` | `note_id`, `project_id` |
| `note_updated` | `note_id` |
| `project_switched` | `project_id` |
| `agent_query` | truncated prompt hash (no full text in v1 log) |

Events store `occurred_at` (UTC) and optional `duration_ms` where applicable.

### Time on task (optional)

- User can start/stop a **timer** on one task at a time
- Timer sessions stored: `task_id`, `started_at`, `ended_at`, `duration_ms`
- Running timer visible in shell status area

### Summaries

| View | Content |
|------|---------|
| **Today** | Tasks completed, notes touched, time logged |
| **Week** | Same aggregates + simple bar by day |

Summaries are computed from activity + timer tables (no separate rollup table in v1).

## Non-goals (v1)

- Pomodoro mode
- Export to CSV/JSON (deferred to polish phase)
- AI-generated daily standup (deferred; agent can read log via tools in v1.1)

## Related plans

- [swift-v1](../plan/swift-v1.md) — phase 6 polish
- [swift-data-v1](../plan/swift-data-v1.md)
