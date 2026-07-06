# Swift — product overview

**Status:** Planned

## Scope

Swift is a **personal** project management and **knowledge hub**. It supports **multiple projects**, tracks tasks, ingests notes/emails/Slack/docs per project, and provides an **AI assistant** (Ollama + vector search) over that knowledge.

## Personas

| Persona | Needs |
|---------|-------|
| **Solo builder** | Track projects, search all context (notes, mail, chat), ask AI across sources |
| **Nest contributor** | Dogfood Tauri + React; manage framework work with semantic search over docs |

## Vision

1. **Project management** — projects, tasks, kanban/list
2. **Knowledge** — unified store for notes, emails, Slack, documentation per project
3. **AI** — agent with **vector search** + MCP tools; answers from project knowledge, not just chat memory

## Success criteria (v1)

- Multi-project workspace with task management
- Notes and project docs stored in PostgreSQL with **pgvector** embeddings
- Agent searches project knowledge semantically (`swift_search_knowledge`)
- PostgreSQL + pgvector required; Ollama for chat; embedding API for index
- Reusable UI patterns documented for desktop template

## Non-goals (v1)

- Team accounts / multi-user sync
- Real-time email/Slack sync (batch ingest OK in v1.1)
- Mobile or web clients
- Replacing Cursor as coding agent host

## Related plans

- [swift-v1](../plan/swift-v1.md)
- [nest-data-postgres v1](../../../../docs/plan/nest-data-postgres-v1.md)
