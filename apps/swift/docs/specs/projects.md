# Swift — projects

**Status:** Planned

## Scope

Defines the **project** entity and multi-project workspace behavior.

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
| `created_at` / `updated_at` | yes | UTC timestamps |

### Workspace

- User maintains an ordered list of **active** projects
- **Archived** projects are hidden from default switcher but recoverable
- Exactly one **active project** context at a time drives task/note default scope
- Global view (all projects) available for search and task filters

### Project switcher (UI)

- Accessible from shell sidebar header
- Shows name, color dot, optional icon
- Keyboard shortcut to open (defined in [ui-platform](../specs/ui-platform.md))

### Per-project settings

- Default task statuses (optional override of workspace defaults)
- Default note folder (optional)
- Archive / restore / delete (delete requires confirmation; cascades per [data-model](data-model.md))

## Data sketch

```text
projects
  id, slug, name, description, color, icon, archived, sort_order, created_at, updated_at
```

## Non-goals (v1)

- Project templates
- Shared/team projects
- Git repo auto-linking (deferred)

## Related plans

- [swift-data-v1](../plan/swift-data-v1.md)
- [swift-pm-v1](../plan/swift-pm-v1.md)
