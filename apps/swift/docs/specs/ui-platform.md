# Swift — UI platform

**Status:** Planned

## Scope

React + Tailwind shell layout and design conventions. Swift **dogfoods** [`templates/desktop/`](../../../../templates/desktop/) and feeds improvements back ([swift-template-feedback-v1](../plan/swift-template-feedback-v1.md)).

## Stack

| Layer | Choice |
|-------|--------|
| Host | Tauri 2 + `nest-tauri` |
| UI | React 19 + TypeScript |
| Styling | Tailwind 3 + `nest-react-theme` preset |
| Icons | Lucide React |
| Routing | React Router (v1) |

## Shell layout

```text
┌─────────────────────────────────────────────────────────────┐
│ Title bar (Tauri)                                           │
├──────────┬──────────────────────────────────┬───────────────┤
│ Sidebar  │ Main content                     │ Agent rail    │
│          │                                  │ (collapsible) │
│ Project  │  Tasks / Notes / Settings        │               │
│ switcher │                                  │ Chat + tools  │
│ Nav      │                                  │               │
│          │                                  │               │
└──────────┴──────────────────────────────────┴───────────────┘
```

### Sidebar

- Project switcher (top)
- Nav: Tasks, Knowledge, Activity (phase 6), Settings
- Collapse to icons on narrow width

### Main

- Route-driven views per project context
- Breadcrumb: Project name → view

### Agent rail

- Toggle button in shell header
- Resizable width (localStorage pref)

## Design tokens

Use CSS variables from `nest_theme_css` + Tailwind `nest-*` utilities. No hard-coded palette in components.

## Shared components (candidates for template)

| Component | Used in |
|-----------|---------|
| `AppShell` | All views |
| `ProjectSwitcher` | Sidebar |
| `KanbanBoard` / `TaskCard` | Tasks |
| `TaskList` | Tasks |
| `MarkdownEditor` | Notes |
| `AgentPanel` | Agent rail |
| `SearchCommand` | Global ⌘K (deferred v1.1) |

Document each promotion in [swift-template-feedback-v1](../plan/swift-template-feedback-v1.md).

## Non-goals (v1)

- `@nest/ui` npm package (extract in phase 5)
- Dark/light toggle beyond nest-theme (follow system via ThemeModule)

## Related plans

- [swift-scaffold-v1](../plan/swift-scaffold-v1.md)
- [swift-pm-v1](../plan/swift-pm-v1.md)
- [swift-template-feedback-v1](../plan/swift-template-feedback-v1.md)
