# Swift collector + knowledge AI v1

## Status: In progress (P1 extract+import IPC done; search UI next)

Inspiration: [AnythingLLM](https://anythingllm.com/) and especially its
[`collector`](https://github.com/Mintplex-Labs/anything-llm/tree/master/collector)
pipeline — **ingest → normalize → chunk → embed → retrieve → answer**.

This plan maps that pipeline onto Nest + Swift’s existing stack:

- **Chat / tools:** Ollama on `server.lan` via `nest-ai-ollama` + `nest-agent`
- **Embeddings:** Ollama `nomic-embed-text` (768-d) — already in `config.toml`
- **Store:** PostgreSQL + pgvector (`knowledge_articles` / `knowledge_items`)
- **UX:** Tauri agent panel + knowledge UI (not AnythingLLM’s full product shell)

It **extends** [swift-knowledge-v1](./swift-knowledge-v1.md) and
[swift-agent-v1](./swift-agent-v1.md); it does **not** replace them.

## Specs

- [knowledge](../specs/knowledge.md)
- [ai-assistant](../specs/ai-assistant.md)
- [ipc-api](../specs/ipc-api.md)

## Why AnythingLLM’s collector (and what we copy)

### What the collector is

A standalone **Node/Express document processor** (~portable microservice). The
AnythingLLM server uploads / drops files into `hotdir/`, then calls collector
HTTP endpoints. Collector’s job is **text extraction only** — embeddings and
chat happen elsewhere.

| Endpoint | Role |
|----------|------|
| `POST /process` | File in hotdir → typed converter → document JSON |
| `POST /parse` | Same as process, but `parseOnly` (no persist side-effect) |
| `POST /process-link` | Fetch URL → text/HTML → document |
| `POST /process-raw-text` | Pasted text + metadata → document |
| `GET /accepts` | MIME / extension allow-list |
| `extensions/*` | YouTube, GitHub, Confluence, etc. |

Per-file converters live under
[`processSingleFile/convert/`](https://github.com/Mintplex-Labs/anything-llm/tree/master/collector/processSingleFile/convert):

| Converter | Formats |
|-----------|---------|
| `asTxt` | `.txt`, `.md`, `.csv`, `.json`, `.html`, … |
| `asPDF` | `.pdf` |
| `asDocx` | `.docx` (mammoth) |
| `asOfficeMime` | `.pptx`, `.odt`, `.odp` |
| `asXlsx` | `.xlsx` |
| `asEPub` / `asMbox` / `asAudio` / `asImage` | media / mail (later) |

Each converter emits a **normalized document record**, roughly:

```text
id, title, url, docAuthor, description, docSource,
pageContent, wordCount, token_count_estimate, metadata…
```

That shape is the bridge into embedding + retrieval.

### What we do **not** copy

| AnythingLLM | Nest / Swift |
|-------------|--------------|
| Separate Node collector + Electron desktop | In-process Tauri commands (preferred); optional sidecar later |
| LangChain / js-tiktoken in collector | Rust chunker + Ollama tokenize-by-chars/heuristic |
| Its own vector DB / workspace UX | pgvector + Swift Knowledge UI |
| Multi-tenant hosted cloud | Personal desktop, one PostgreSQL (`swift` DB) |
| Built-in LLM marketplace | Our Ollama server only |

**Rule:** steal the **pipeline vocabulary and format coverage**, implement in
Nest crates / Swift services.

## Target product behaviors (your focus)

| Capability | How it works in Swift |
|------------|------------------------|
| **Review documents** | Import/upload → collector extract → store `kind=doc` → open in Knowledge view; agent can summarize / critique via RAG |
| **Ask about processes** | RAG over project docs + notes (“how do we deploy?”, “what’s the onboarding checklist?”) |
| **Create docs from templates** | Template library + agent fill (`swift_render_template` + optional LLM expansion) → new note/doc |
| **PM helpers** | Existing task tools (`swift_search_tasks`, create/update behind `allow_writes`) |

## Architecture

```text
                 ┌──────────────────────────────────────────┐
  UI (React)     │ Knowledge ingest UI │ Agent panel        │
                 └───────────┬───────────────────┬──────────┘
                             │ Tauri IPC         │ Tauri IPC
                 ┌───────────▼───────────────────▼──────────┐
  src-tauri      │ KnowledgeService │ AgentRunner           │
                 │ EmbeddingService │ CompositeToolSource   │
                 └───────────┬───────────────────┬──────────┘
                             │                   │
         ┌───────────────────┼───────────────────┼────────────┐
         ▼                   ▼                   ▼            ▼
   DocumentCollector    PostgreSQL          nest-ai-ollama   Ollama
   (extract+chunk)      + pgvector          chat + embed     :11434
         │                   ▲                   │
         └───── pageContent ─┴── embedding ──────┘
```

### Nest / Swift crates (proposed)

| Crate / module | Responsibility |
|----------------|----------------|
| `swift-collector` (new, `apps/swift/crates/`) **or** `nest-collector` (if we want reuse) | Format extractors + chunking → `CollectedDocument` |
| `swift-data` | `KnowledgeRepository`, similarity search |
| `EmbeddingService` | Ollama `/api/embed` on ingest + query |
| `swift-agent-tools` | `swift_search_knowledge`, task tools, template tools |
| `nest-agent` + `nest-ai-ollama` | Multi-step tool loop, streaming chat |

**Recommendation:** start as **`swift-collector`** inside the Swift app. Promote
to `modules/crates/nest-collector` once a second app needs the same extractors.

## Pipeline (AnythingLLM-shaped)

### Stage 1 — Collect (extract)

Input sources (v1 priority):

1. File picker / drag-drop (PDF, DOCX, MD, TXT, HTML, CSV, XLSX, PPTX)
2. Paste raw text
3. Import URL (HTML → text; optional)
4. Author a note in-app (already planned)

Output: `CollectedDocument { title, source_uri, mime, text, metadata }`.

v1 extractors (Rust-first):

| Format | Approach |
|--------|----------|
| `.md` / `.txt` / `.csv` / `.json` / `.html` | Read UTF-8; strip HTML with a small parser |
| `.pdf` | **`pdf-extract`** crate (`extract_text_from_mem`) |
| `.docx` | ZIP + `word/document.xml` text runs (`w:t`) |
| `.xlsx` | Later / optional (CSV export path first) |

Defer (AnythingLLM has them; we wait): audio/OCR/EPUB/mbox/YouTube until a
real need appears.

### Stage 2 — Chunk

AnythingLLM often embeds whole docs or splits downstream. For long process
docs we need explicit chunking:

| Setting | Default |
|---------|---------|
| Chunk size | ~800–1200 tokens (~3–4k chars) |
| Overlap | ~10–15% |
| Strategy | Paragraph-aware, then hard split |

Store **chunks** as rows (or child rows) with parent `knowledge_article_id`,
`chunk_index`, `embedding vector(768)`.

### Stage 3 — Embed (Ollama)

```toml
[ollama]
base_url = "http://192.168.88.10:11434"

[embeddings]
provider = "ollama"
model = "nomic-embed-text"
dimensions = 768
```

On collect success:

1. Insert article + revision
2. Chunk text
3. `POST /api/embed` per chunk (batch if Ollama supports)
4. Upsert embeddings; set `indexed_at`

Re-index on note/doc edit.

### Stage 4 — Retrieve + answer (agent)

User asks in Agent panel → `swift_search_knowledge(query)` → top-k chunks →
inject into agent context (or agent tool returns snippets) → Ollama chat model
(`qwen2.5-coder:*` or a stronger instruct model for Q&A).

For process Q&A, prefer a **non-tiny** chat model (your 3b default is fine for
smoke; upgrade for quality).

### Stage 5 — Templates → documents

| Piece | Behavior |
|-------|----------|
| Template store | Markdown files under `apps/swift/templates/` **or** DB table `doc_templates` |
| Variables | `{{project_name}}`, `{{date}}`, checklist sections |
| Agent tool | `swift_list_templates`, `swift_render_template` → creates `kind=note|doc` |
| Optional LLM pass | “Expand this outline for project X using our deploy process” (RAG + template) |

Wire to [swift-template-feedback-v1](./swift-template-feedback-v1.md) only for
**UI chrome**; product doc templates stay Swift-owned.

## Phased delivery

Aligned with Swift master phases; collector work sits mainly in **3** and **4**.

### P0 — Prerequisites (blocking)

| Item | Status / action |
|------|-----------------|
| Phase 0 scaffold | Done |
| Phase 1 data + pgvector | Done — [swift-data-v1](./swift-data-v1.md) |
| Phase 2 PM | Required for agent task tools — [swift-pm-v1](./swift-pm-v1.md) |
| Ollama on server | Chat + `nomic-embed-text` pulled |
| PostgreSQL on server | `config.toml` already points at `192.168.88.10` |

### P1 — Collector MVP (knowledge ingest)

Deliverable for [swift-knowledge-v1](./swift-knowledge-v1.md) **3d expanded**:

1. `CollectedDocument` + extractors (txt/md/html first, then PDF/DOCX) — **done** (`swift-collector`: txt/md/html/csv/json/pdf/docx)
2. Tauri: `swift_import_doc`, `swift_import_raw_text`, `swift_collector_accepts` — **done**
3. Chunk + embed via Ollama — **done** (`knowledge_chunks` + article embed)
4. Knowledge list + semantic search UI — **done** (Hybrid / Semantic / Keyword modes → `swift_search_knowledge`)
5. Unit tests with fixture files (no live Ollama required for extract tests) — **done**
6. Knowledge UI import — **done** (ribbon **Import Doc** + header button → `swift_pick_import_doc` → extract/index)

**Done when:** drop a PDF/MD into a project → searchable by meaning.

### P2 — Agent Q&A (project assistant)

Extend [swift-agent-v1](./swift-agent-v1.md):

| Step | Deliverable |
|------|-------------|
| 4a | Ollama chat smoke (no tools) — **done** (stream via nest-ai-ollama) |
| 4b | Optional MCP (keep off by default) — deferred |
| 4c | `swift_search_knowledge` + cite snippets in reply — **done** (RAG retrieve → citations → stream) |
| 4d | Task tools + gated writes — next |
| 4e | Streaming + tool UI — **done** (basic) |

**System prompt roles:** “Swift project assistant — answer from indexed project
knowledge; say when you don’t know; prefer processes and task state over
guessing.”

**Done when:** “What is our release process?” answers from imported docs with
citations (title + snippet).

### P3 — Templates + doc generation

1. Seed 3–5 templates (process SOP, meeting notes, project brief, retro) — **done** (`apps/swift/templates/`)
2. UI: New from template — **done** (Knowledge ribbon → From Template)
3. Agent tools to render + optionally expand with RAG context — **done** (intent match → `swift_render_template`; RAG expand deferred)
4. `allow_writes` gates creating notes/tasks — **done** (agent template + task create/update; UI “From Template” remains user-initiated)

**Done when:** “Create a project brief for Acme using the standard template”
produces a saved note. — **met** (agent intent path + UI)

IPC: `swift_list_templates`, `swift_render_template` (`persist` default true).

### P4 — Richer collectors (optional / v1.1)

Mirror AnythingLLM extensions selectively:

- URL ingest polish
- Confluence / Notion / GitHub README connector (if you need them)
- Email / Slack batch import (already deferred in knowledge spec)

## IPC sketch (additions)

| Command | Purpose |
|---------|---------|
| `swift_collector_accepts` | Supported extensions / MIME |
| `swift_import_doc` | Path or bytes → collect → index |
| `swift_import_raw_text` | Paste → index |
| `swift_import_url` | Optional URL ingest |
| `swift_reindex_knowledge` | Rebuild embeddings for a project |
| `swift_list_templates` / `swift_render_template` | Template library |

Agent events: reuse planned `swift://agent/*` stream.

## Config (already mostly there)

```toml
[ollama]
base_url = "http://192.168.88.10:11434"
default_model = "qwen2.5-coder:3b"   # raise for better Q&A later

[embeddings]
provider = "ollama"
model = "nomic-embed-text"
dimensions = 768

[agent]
max_steps = 8
allow_writes = false

[collector]
# optional future: max_upload_mb, chunk_size, chunk_overlap
```

## Risks

| Risk | Mitigation |
|------|------------|
| PDF/DOCX quality in Rust weaker than AnythingLLM’s Node stack | Start MD/TXT; use `pdftotext` binary; evaluate binding Node sidecar only if quality blocks |
| Tiny chat model hallucinates processes | Require RAG citations; fail closed when retrieval empty; upgrade model |
| Embedding dim mismatch | Lock 768; document full re-index on model change |
| Phase 1 data not ready | Unblocked — `swift-data-v1` implemented |
| Scope creep (full AnythingLLM clone) | Non-goals below |

## Non-goals (this plan)

- Forking / vendoring AnythingLLM collector as a long-term Node dependency
- Multi-user / multi-tenant AnythingLLM Server mode
- Shipping an Electron AnythingLLM desktop beside Swift
- OCR / audio transcription in v1
- Replacing Nest MCP memory tools with AnythingLLM Community Hub skills

## Relationship to existing plans

| Existing | Change |
|----------|--------|
| [swift-knowledge-v1](./swift-knowledge-v1.md) | Expand 3d “doc import” into full collector pipeline |
| [swift-agent-v1](./swift-agent-v1.md) | Emphasize RAG citations + template tools |
| [swift-v1](./swift-v1.md) | Link this as the collector / RAG companion plan |
| [knowledge spec](../specs/knowledge.md) | Already matches; no contradiction |

## Implementation order (recommended)

```text
1. swift-data-v1 ✓ (PG + knowledge tables + pgvector + Ollama embed)
2. collector extractors (txt/md) + import IPC + embed ✓
3. semantic search UI ✓
4. nest-agent chat + swift_search_knowledge ✓ (RAG citations + stream)
5. PDF/DOCX extractors ✓ (`pdf-extract` + DOCX ZIP/`document.xml`)
6. templates + agent render tools ✓
7. PM write tools behind allow_writes ✓ (`swift_create_task` / `swift_update_task` + template create gated)
```

## Done when

1. User can ingest project process docs (MD/PDF/DOCX) into a project
2. Agent answers process questions **from those docs** (not only training data)
3. User can create a document from a template (manual UI + agent-assisted)
4. Agent can help with tasks (search / optional create) gated by config
5. All of the above run against Ollama on `192.168.88.10` and PostgreSQL on the same server

## Related

- [AnythingLLM](https://anythingllm.com/)
- [AnythingLLM collector](https://github.com/Mintplex-Labs/anything-llm/tree/master/collector)
- [swift-knowledge-v1](./swift-knowledge-v1.md)
- [swift-agent-v1](./swift-agent-v1.md)
- [swift-v1](./swift-v1.md)
- [nest-ai-ollama](../../../docs/nest-ai-ollama/README.md)
- [nest-agent](../../../docs/nest-agent/README.md)
