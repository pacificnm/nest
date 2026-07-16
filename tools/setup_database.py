"""Create all Nest memory tables and indexes."""

import sys
from pathlib import Path

from memory_common import PROJECT_ROOT, database_url

CONTEXT_SCHEMA_SQL = """
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS agent_context_memory (
    id bigserial PRIMARY KEY,
    session_key text NOT NULL DEFAULT '',
    title text NOT NULL DEFAULT '',
    content text NOT NULL,
    tags text[] NOT NULL DEFAULT '{}',
    embedding vector(768) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS agent_context_memory_embedding_idx
    ON agent_context_memory
    USING hnsw (embedding vector_cosine_ops);

CREATE INDEX IF NOT EXISTS agent_context_memory_session_created_idx
    ON agent_context_memory (session_key, created_at DESC);

CREATE INDEX IF NOT EXISTS agent_context_memory_created_idx
    ON agent_context_memory (created_at DESC);
"""

SCHEMA_SQL = """
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS project_memory (
    id bigserial PRIMARY KEY,
    source_path text NOT NULL,
    content text NOT NULL,
    content_hash text NOT NULL UNIQUE,
    embedding vector(768) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS project_memory_embedding_idx
    ON project_memory
    USING hnsw (embedding vector_cosine_ops);

CREATE INDEX IF NOT EXISTS project_memory_source_path_idx
    ON project_memory (source_path);
""" + CONTEXT_SCHEMA_SQL.replace("CREATE EXTENSION IF NOT EXISTS vector;\n\n", "", 1) + """

CREATE TABLE IF NOT EXISTS knowledge_base (
    id bigserial PRIMARY KEY,
    collection text NOT NULL,
    source_path text NOT NULL,
    content text NOT NULL,
    content_hash text NOT NULL,
    embedding vector(768) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS kb_hash_idx
    ON knowledge_base (content_hash);

CREATE INDEX IF NOT EXISTS kb_coll_idx
    ON knowledge_base (collection);

CREATE INDEX IF NOT EXISTS kb_embed_idx
    ON knowledge_base
    USING hnsw (embedding vector_cosine_ops);
"""

SQL_FILE = PROJECT_ROOT / "tools" / "setup_database.sql"


def main() -> int:
    import psycopg

    try:
        with psycopg.connect(database_url()) as conn:
            conn.execute(SCHEMA_SQL)
            conn.commit()
    except psycopg.errors.InsufficientPrivilege:
        print(
            "ERROR: current database user cannot create tables in schema public.\n"
            "Run the one-time setup as postgres instead:\n"
            "  ./scripts/setup-database-postgres.sh",
            file=sys.stderr,
        )
        return 1

    print("nest_memory schema ready (project_memory, agent_context_memory, knowledge_base).")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(f"ERROR: database setup failed: {error}", file=sys.stderr)
        sys.exit(1)
