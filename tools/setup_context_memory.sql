-- Agent context memory table (also included in tools/setup_database.sql).
-- Run as postgres if your app user cannot CREATE tables:
--   sed "s/REPLACE_WITH_OS_USER/$USER/g" tools/setup_context_memory.sql | sudo -u postgres psql nest_memory

CREATE EXTENSION IF NOT EXISTS vector;

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

GRANT SELECT, INSERT, UPDATE, DELETE ON agent_context_memory TO "REPLACE_WITH_OS_USER";
GRANT USAGE, SELECT ON SEQUENCE agent_context_memory_id_seq TO "REPLACE_WITH_OS_USER";
