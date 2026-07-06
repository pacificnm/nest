# Swift template feedback v1

## Status: Planned

Phase **5** of [swift-v1](./swift-v1.md). Promote reusable UI from Swift to [`templates/desktop/`](../../../templates/desktop/).

## Specs

- [ui-platform](../specs/ui-platform.md) — component candidates

## Context

Swift dogfoods the desktop template. Patterns proven in Swift should flow back so the next Nest desktop app starts with a richer baseline.

## Promotion process

For each component:

1. Mark **stable** in Swift (used in 2+ views, no churn for one sprint)
2. Generalize props (remove Swift-specific naming)
3. Copy to `templates/desktop/ui/src/components/`
4. Document in template README
5. Swift imports from local copy or shared path until `@nest/ui` exists

## Target promotions (v1)

| Component | Template path | Notes |
|-----------|---------------|-------|
| `AppShell` | `components/AppShell.tsx` | Sidebar + main + optional rail slot |
| `ProjectSwitcher` | — | May stay Swift-specific; extract `Dropdown` primitive |
| `KanbanBoard` | `components/KanbanBoard.tsx` | Generic columns + drag |
| `TaskCard` | `components/KanbanCard.tsx` | Renamed generic card |
| `MarkdownEditor` | `components/MarkdownEditor.tsx` | Simple MD editor |
| `AgentPanel` | `components/AgentPanel.tsx` | Depends on [swift-agent-v1](./swift-agent-v1.md) |
| `useTauriCommand` | `hooks/useTauriCommand.ts` | Typed invoke wrapper |

Minimum for phase 5 **done**: **3** components/hooks promoted (per [swift-v1](./swift-v1.md) success criteria).

## Non-goals (v1)

- Publishing `@nest/ui` npm package
- Extracting Swift Rust crates into nest core

## Done when

- Template README lists promoted components with usage examples
- New app from template includes AppShell + at least one PM or agent primitive
- Swift still builds using promoted files (copy or symlink documented)

## Related

- [nest-react-ui v1](../../../docs/plan/nest-react-ui-v1.md)
- [swift-pm-v1](./swift-pm-v1.md)
- [swift-agent-v1](./swift-agent-v1.md)
