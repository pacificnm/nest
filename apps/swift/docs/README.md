# Swift

Personal **project management**, **knowledge hub**, and **AI assistant** desktop app for Nest.

**Status:** Planned (documentation phase)

**Local app path:** `apps/swift/` (gitignored checkout — see [apps/README.md](../../README.md))

**Stack:** [Tauri + React + TypeScript + Tailwind](../../../docs/architecture.md#desktop-frontend-platform) via [`nest-tauri`](../../../docs/nest-tauri/README.md) and [`templates/desktop/`](../../../templates/desktop/)

Swift is the **reference desktop product** for the Nest frontend platform. It dogfoods the desktop template while building a full personal tracker with Ollama-powered agents ([`nest-agent`](../../../docs/nest-agent/README.md), [`nest-ai-ollama`](../../../docs/nest-ai-ollama/README.md)) and **pgvector** semantic search over project knowledge.

## What Swift does

| Area | Summary |
|------|---------|
| **Projects** | Portfolio at scale — pin, archive, searchable library, daily switching |
| **Tasks** | Gantt / Task Sheet, outline, predecessors, % complete |
| **Knowledge** | Notes, emails, Slack, docs per project — unified store with **vector search** |
| **Tracking** | Personal activity log, optional time-on-task, summaries |
| **AI assistant** | Agent panel with Ollama + MCP + **`swift_search_knowledge`** (vector) |

## App layout

```text
apps/swift/
├── docs/               # specs + plans (this tree)
├── ui/                 # React + TypeScript + Tailwind
├── crates/
│   ├── swift-data/     # repositories on nest-data-postgres
│   └── swift-agent-tools/
└── src-tauri/          # nest-tauri + Nest modules + Swift domain services
```

Specs and plans live in **`apps/swift/docs/`** (this tree), versioned in git and indexed by [project memory](../../../tools/MCP-SETUP.md).

## Specifications

Stable requirements and contracts — *what* the product must do.

| Spec | Topic |
|------|-------|
| [overview](specs/overview.md) | Vision, personas, success criteria |
| [projects](specs/projects.md) | Multi-project model |
| [tasks](specs/tasks.md) | Task/issue tracking |
| [tracking](specs/tracking.md) | Personal activity and time |
| [knowledge](specs/knowledge.md) | Multi-source knowledge, vector search, linking |
| [ai-assistant](specs/ai-assistant.md) | Agent UX, Ollama, MCP, vector search tools |
| [data-model](specs/data-model.md) | PostgreSQL + pgvector schema |
| [ui-platform](specs/ui-platform.md) | Shell layout, design system, template feedback |
| [ipc-api](specs/ipc-api.md) | Tauri commands and events |

## Implementation plans

Phased rollout — *how and when* to build.

| Plan | Phase |
|------|-------|
| [swift-v1](plan/swift-v1.md) | **Master** — phases 0–6, dependencies |
| [swift-scaffold-v1](plan/swift-scaffold-v1.md) | 0 — scaffold from desktop template |
| [swift-data-v1](plan/swift-data-v1.md) | 1 — PostgreSQL data layer via `nest-data-postgres` |
| [swift-pm-v1](plan/swift-pm-v1.md) | 2 — projects + tasks UI |
| [swift-knowledge-v1](plan/swift-knowledge-v1.md) | 3 — knowledge ingest, embeddings, search |
| [swift-agent-v1](plan/swift-agent-v1.md) | 4 — AI assistant |
| [swift-template-feedback-v1](plan/swift-template-feedback-v1.md) | 5 — feed shared UI back to template |

## Related

- [Desktop frontend platform](../../../docs/architecture.md#desktop-frontend-platform)
- [Desktop template](../../../templates/desktop/)
- [nest-data-postgres v1](../../../docs/plan/nest-data-postgres-v1.md)
- [nest-agent + MCP plan](../../../docs/plan/nest-agent-mcp-v1.md)
- [nest-data](../../../docs/nest-data/README.md)
