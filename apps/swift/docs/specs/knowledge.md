# Swift — knowledge base

**Status:** Planned

## Scope

**Project-scoped knowledge** — notes, emails, Slack messages, and project documentation — stored in PostgreSQL, indexed for **vector search** so the AI assistant can answer questions from all sources.

## Content types

| Kind | v1 | Description |
|------|-----|-------------|
| `note` | yes | User-authored markdown (editor UI) |
| `email` | planned | Imported messages (IMAP/API ingest deferred to v1.1+) |
| `slack` | planned | Imported channel/DM messages |
| `doc` | yes | Project docs, specs, pasted documentation |

All types map to [`knowledge_articles`](database-schema.md) with categories and revision history.

## Note editor (UI)

Markdown notes are the primary authoring surface in v1:

| Field | Required |
|-------|----------|
| `title`, `body` | yes |
| `project_id` | yes (defaults to focus project; omit for workspace search) |
| Folder organization | optional (v1.1) |

On save, note content syncs to `knowledge_articles` (`source_type = manual`) and appends a `knowledge_revisions` row; embedding refresh follows.

## Linking

- Attach knowledge articles to tasks via `task_knowledge_links`
- Wiki-style `[[links]]` parsed on save for backlinks (notes only in v1)

## Search

### Vector search (primary)

- **pgvector** similarity on `knowledge_articles.embedding`
- Default: **focus project**; workspace-wide when user or agent omits `project_id`
- Used by AI assistant and in-app “semantic search”

### Keyword search (secondary)

- PostgreSQL `tsvector` on title + body for exact terms and hybrid rerank
- Complements vector search for IDs, error codes, names

### UI

- Knowledge view: filter by `kind`, text search bar
- Results show title, kind badge, snippet, similarity score

## Ingestion (phased)

| Source | Phase |
|--------|-------|
| Manual notes | v1 (phase 3) |
| File/doc import (text/markdown) | v1 |
| Email | v1.1+ |
| Slack | v1.1+ |

Ingest pipeline: normalize → insert `knowledge_articles` + revision → generate embedding (Ollama `/api/embed`) → update `indexed_at`.

## Embeddings

| Setting | Default |
|---------|---------|
| Provider | **Ollama** (same server as chat) |
| Model | `nomic-embed-text` (`ollama pull` on server) |
| Dimensions | **768** — must match `vector(N)` in PostgreSQL |
| API | `POST {ollama.base_url}/api/embed` |

Query-time search embeds the user question with the **same model** as indexing. Optional `provider = "openai"` in config for cloud embeddings (1536 dims; requires full re-index).

## Non-goals (v1)

- Real-time Slack/email sync (batch import OK)
- Attachment binary storage (text extraction only; files on disk via nest-file later)
- Published/static site export
- Multi-user collaboration

## Related plans

- [swift-knowledge-v1](../plan/swift-knowledge-v1.md)
- [swift-data-v1](../plan/swift-data-v1.md)
- [nest-data-postgres v1](../../../../docs/plan/nest-data-postgres-v1.md)
