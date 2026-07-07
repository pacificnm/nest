# Swift — product overview

**Status:** Planned

## Scope

Swift is a **personal** project management and **knowledge hub**, with a **Microsoft Project–familiar** UI (ribbon, Gantt Chart, Task Sheet). It supports a large project portfolio, per-project scheduling, knowledge per project, and an AI assistant with vector search.

## Personas

| Persona | Needs |
|---------|-------|
| **Solo builder** | Jump between several projects daily; archive finished work; search across portfolio |
| **Nest contributor** | Dogfood Tauri + React; manage framework work with semantic search over docs |

## Vision

1. **Project portfolio** — Project Center; many projects; archive-first
2. **Scheduling** — Gantt Chart, Task Sheet, predecessors, % complete (MS Project model)
3. **Knowledge** — notes/docs per project (Swift extension via View → Notes)
4. **AI** — agent with vector search + MCP tools

## Success criteria (v1)

- Portfolio-scale workspace (pinned, recents, archive, searchable library)
- Notes and project docs stored in PostgreSQL with **pgvector** embeddings
- Agent searches project knowledge semantically (`swift_search_knowledge`)
- PostgreSQL + pgvector on **remote server**; **Ollama** on server for chat **and embeddings**; OpenAI embeddings optional
- Reusable UI patterns documented for desktop template

## Non-goals (v1)

- Team accounts / multi-user sync
- Real-time email/Slack sync (batch ingest OK in v1.1)
- Mobile or web clients
- Replacing Cursor as coding agent host

## Related plans

- [swift-v1](../plan/swift-v1.md)
- [nest-data-postgres v1](../../../../docs/plan/nest-data-postgres-v1.md)
