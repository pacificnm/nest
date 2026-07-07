# Swift — projects

**Status:** Planned

## Scope

Defines the **project** entity and multi-project workspace behavior. Swift is a **project portfolio** (hundreds of projects over a career, many archived), not a single mega backlog. Users work across **multiple projects daily** — navigation optimizes for quick switching among a small active set, not one deep project context.

## Requirements

### Project entity

Each project has:

| Field | Required | Notes |
|-------|----------|-------|
| `id` | yes | Stable UUID |
| `name` | yes | Display name |
| `slug` | yes | URL-safe key, unique |
| `description` | no | Short summary |
| `color` | no | Accent for UI (hex or token name) |
| `icon` | no | Lucide icon name |
| `archived` | yes | Default `false` |
| `pinned` | no | Default `false`; pinned projects surface in sidebar / daily set |
| `created_at` / `updated_at` | yes | UTC timestamps |

### Workspace

- Portfolio grows to **100s of projects** over time; **archive is the normal end state** for finished work
- **Active** projects are the small set you touch regularly; most rows are archived but searchable
- **Pinned** projects (user-chosen) plus **recents** (last opened, client-side OK in v1) form the daily working set
- **Focus project** — current route scope for tasks/knowledge (`/p/:slug/…`); user switches often within a session
- **Global views** — cross-project task filters and workspace knowledge search (see [tasks](tasks.md), [data-model](data-model.md))

### Project switcher (UI)

- **Searchable command palette** — typeahead over active + archived; not a flat list of every project
- Sidebar shows **pinned** projects and **recents** only (typically &lt; 10 items)
- “All projects…” opens full **project library** (search, filter archived, pin/unpin)
- Keyboard shortcut to open switcher (defined in [ui-platform](ui-platform.md))
- Shows name, color dot, optional icon

### Per-project settings

- Default task statuses (optional override of workspace defaults)
- Default note folder (optional)
- Archive / restore / delete (delete requires confirmation; cascades per [data-model](data-model.md))

## Data sketch

```text
projects
  id, slug, name, description, color, icon, archived, pinned, sort_order, created_at, updated_at
```

## Non-goals (v1)

- Project templates
- Shared/team projects
- Git repo auto-linking (deferred)

## Related plans

- [swift-data-v1](../plan/swift-data-v1.md)
- [swift-pm-v1](../plan/swift-pm-v1.md)
