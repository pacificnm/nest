# Swift — UI platform

**Status:** In progress (scaffold)

## Scope

React + Tailwind shell modeled on **Microsoft Project** (ribbon, Gantt-first workspace, status bar). Swift **dogfoods** [`templates/desktop/`](../../../../templates/desktop/) and feeds improvements back ([swift-template-feedback-v1](../plan/swift-template-feedback-v1.md)).

Portfolio scale (many projects) uses **Project Center** (File / Project ribbon) — not a persistent sidebar.

## Stack

| Layer | Choice |
|-------|--------|
| Host | Tauri 2 + `nest-tauri` |
| UI | React 19 + TypeScript |
| Styling | Tailwind 3 + `nest-react-theme` preset |
| Icons | Lucide React |
| Routing | React Router (v1) |

## Shell layout (MS Project–style)

```text
┌─────────────────────────────────────────────────────────────┐
│ Title bar (Tauri)                                           │
├─────────────────────────────────────────────────────────────┤
│ [Save][Undo][Redo]     Project Name              [Agent]    │
│ File | Task | View | Project | Help                         │
│ Ribbon panels (contextual groups)                           │
│ Active view label (e.g. Gantt Chart)                        │
├──────────────────────────────────────────────┬──────────────┤
│ Entry table │ Timeline / Gantt               │ Agent rail   │
│ (Task Name, Duration, Start, Finish, …)      │ (optional)   │
├──────────────────────────────────────────────┴──────────────┤
│ New Tasks: 0 │ status message…          │ Gantt Chart │100%│
└─────────────────────────────────────────────────────────────┘
```

### Quick Access Toolbar

- Save, Undo, Redo (top row; wired in later phases)

### Ribbon

Tabs mirror Project: **File**, **Task**, **View**, **Project**, **Help**

| Tab | Groups (v1 scaffold) |
|-----|----------------------|
| File | New, Open, Save; Close, Exit |
| Task | Schedule; Tasks (New, Delete, Indent, Outdent); Properties |
| View | Task Views (Gantt, Task Sheet, Calendar, Network); Notes; Show/Hide |
| Project | Project Information; Project Center |
| Help | About |

### Main workspace

- **Gantt Chart** (default): ~48% entry grid + timeline with timescale header
- **Task Sheet**: full-width grid, same columns
- **Project Center**: open/pin/archive portfolio (phase 2)
- No left sidebar — navigation via ribbon and views

### Status bar

Three zones (like Project):

| Zone | Content |
|------|---------|
| Left | `New Tasks: N` |
| Center | Transient messages via `useStatusBar()` |
| Right | Active view name + zoom % |

### Agent rail

Swift-specific extension — toggle from ribbon **View** or quick-access row. Not part of MS Project.

## Design tokens

Use CSS variables from `nest_theme_css` + Tailwind `nest-*` utilities. No hard-coded palette in components.

## Shared components

| Component | Role |
|-----------|------|
| `AppShell` | Full MS Project–style frame |
| `ProjectRibbon` / `Ribbon` | Tabs + ribbon groups |
| `GanttChartView` | Entry grid + Gantt split |
| `TaskSheetView` | Grid-only view |
| `StatusBar` | Three-zone footer |
| `ProjectCenter` | Portfolio browser (phase 2) |
| `AgentPanel` | Agent rail (phase 4) |

## Non-goals (v1)

- Full Office backstage for File tab
- Resource / Report ribbon tabs
- `@nest/ui` npm package (extract in phase 5)

## Related plans

- [swift-scaffold-v1](../plan/swift-scaffold-v1.md)
- [swift-pm-v1](../plan/swift-pm-v1.md)
- [tasks](tasks.md)
