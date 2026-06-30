"""Semantic search against knowledge_base."""

from __future__ import annotations

import sys

from embedding import embed_text
from memory_common import database_url


def search_knowledge(
    query: str,
    *,
    limit: int = 8,
    collection: str = "",
) -> list[tuple[str, str, str]]:
    """Return (collection, source_path, content) rows ordered by similarity."""
    import psycopg

    embedding = embed_text(query)
    collection = collection.strip()

    with psycopg.connect(database_url()) as conn:
        if collection:
            rows = conn.execute(
                """
                SELECT collection, source_path, content
                FROM knowledge_base
                WHERE collection = %s
                ORDER BY embedding <=> %s::vector
                LIMIT %s
                """,
                (collection, embedding, limit),
            ).fetchall()
        else:
            rows = conn.execute(
                """
                SELECT collection, source_path, content
                FROM knowledge_base
                ORDER BY embedding <=> %s::vector
                LIMIT %s
                """,
                (embedding, limit),
            ).fetchall()

    return rows


def list_collections() -> list[str]:
    import psycopg

    with psycopg.connect(database_url()) as conn:
        rows = conn.execute(
            """
            SELECT DISTINCT collection
            FROM knowledge_base
            ORDER BY collection
            """
        ).fetchall()
    return [row[0] for row in rows]


def format_results(
    query: str,
    rows: list[tuple[str, str, str]],
    *,
    collection: str = "",
) -> str:
    if not rows:
        return "No matching knowledge base results found."

    header = f"[{len(rows)} results for \"{query}\""
    if collection:
        header += f" in '{collection}'"
    header += "]\n\n"

    blocks = []
    for coll, source_path, content in rows:
        preview = content if len(content) <= 2000 else content[:2000] + "..."
        blocks.append(f"--- {coll}/{source_path} ---\n{preview}")

    return header + "\n\n".join(blocks)


def main() -> int:
    import argparse

    parser = argparse.ArgumentParser(description="Search the Nest knowledge base")
    parser.add_argument("query", nargs="*", help="Search query")
    parser.add_argument("--limit", type=int, default=8)
    parser.add_argument("--collection", default="")
    parser.add_argument("--list-collections", action="store_true")
    args = parser.parse_args()

    try:
        if args.list_collections:
            names = list_collections()
            if not names:
                print("No indexed collections.")
            else:
                for name in names:
                    print(name)
            return 0

        query = " ".join(args.query).strip()
        if not query:
            print('Usage: .venv/bin/python tools/search_knowledge.py "your query"', file=sys.stderr)
            return 1

        rows = search_knowledge(query, limit=args.limit, collection=args.collection)
        print(format_results(query, rows, collection=args.collection))
        return 0
    except Exception as error:
        print(f"ERROR: knowledge search failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
