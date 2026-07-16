import sys

from embedding import embed_text
from memory_common import database_url


query = " ".join(sys.argv[1:]).strip()

if not query:
    print('Usage: .venv/bin/python tools/search_memory.py "your query"')
    sys.exit(1)

try:
    import psycopg

    embedding = embed_text(query)

    with psycopg.connect(database_url()) as conn:
        rows = conn.execute(
            """
            SELECT source_path, content
            FROM project_memory
            ORDER BY embedding <=> %s::vector
            LIMIT 8
            """,
            (embedding,),
        ).fetchall()
except Exception as error:
    print(f"ERROR: memory search failed: {error}", file=sys.stderr)
    sys.exit(1)

for source_path, content in rows:
    print(f"\n--- {source_path} ---\n")
    print(content[:2000])
