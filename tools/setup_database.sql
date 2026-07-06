-- One-time Nest memory database setup.
-- Run as the PostgreSQL superuser, for example:
--   sudo -u postgres createdb nest_memory
--   sudo -u postgres psql nest_memory -f tools/setup_database.sql
--
-- Before running, replace REPLACE_WITH_OS_USER below with your PostgreSQL role
-- (typically your OS username: echo $USER).

CREATE EXTENSION IF NOT EXISTS vector;

-- Indexed repository documentation (README, docs/, AGENTS.md, etc.)
CREATE TABLE IF NOT EXISTS project_memory (
    id bigserial PRIMARY KEY,
    source_path text NOT NULL,
    content text NOT NULL,
    content_hash text NOT NULL UNIQUE,
    embedding vector(1536) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS project_memory_embedding_idx
    ON project_memory
    USING hnsw (embedding vector_cosine_ops);

CREATE INDEX IF NOT EXISTS project_memory_source_path_idx
    ON project_memory (source_path);

-- Agent session checkpoints across Cursor compaction
CREATE TABLE IF NOT EXISTS agent_context_memory (
    id bigserial PRIMARY KEY,
    session_key text NOT NULL DEFAULT '',
    title text NOT NULL DEFAULT '',
    content text NOT NULL,
    tags text[] NOT NULL DEFAULT '{}',
    embedding vector(1536) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS agent_context_memory_embedding_idx
    ON agent_context_memory
    USING hnsw (embedding vector_cosine_ops);

CREATE INDEX IF NOT EXISTS agent_context_memory_session_created_idx
    ON agent_context_memory (session_key, created_at DESC);

CREATE INDEX IF NOT EXISTS agent_context_memory_created_idx
    ON agent_context_memory (created_at DESC);

-- External reference docs (Rust book, Tauri, React, Tailwind, etc.) — indexer deferred
CREATE TABLE IF NOT EXISTS knowledge_base (
    id bigserial PRIMARY KEY,
    collection text NOT NULL,
    source_path text NOT NULL,
    content text NOT NULL,
    content_hash text NOT NULL,
    embedding vector(1536) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS kb_hash_idx
    ON knowledge_base (content_hash);

CREATE INDEX IF NOT EXISTS kb_coll_idx
    ON knowledge_base (collection);

CREATE INDEX IF NOT EXISTS kb_embed_idx
    ON knowledge_base
    USING hnsw (embedding vector_cosine_ops);

-- Grants: replace REPLACE_WITH_OS_USER with your DB role before running.
GRANT SELECT, INSERT, UPDATE, DELETE ON project_memory TO "REPLACE_WITH_OS_USER";
GRANT USAGE, SELECT ON SEQUENCE project_memory_id_seq TO "REPLACE_WITH_OS_USER";

GRANT SELECT, INSERT, UPDATE, DELETE ON agent_context_memory TO "REPLACE_WITH_OS_USER";
GRANT USAGE, SELECT ON SEQUENCE agent_context_memory_id_seq TO "REPLACE_WITH_OS_USER";

GRANT SELECT, INSERT, UPDATE, DELETE ON knowledge_base TO "REPLACE_WITH_OS_USER";
GRANT USAGE, SELECT ON SEQUENCE knowledge_base_id_seq TO "REPLACE_WITH_OS_USER";
