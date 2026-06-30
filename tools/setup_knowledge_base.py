"""Create the knowledge_base table and indexes.

Prefer tools/setup_database.py for full schema setup.
"""

import sys

from memory_common import database_url

from setup_database import SCHEMA_SQL, SQL_FILE


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
            f"  sudo -u postgres psql nest_memory < {SQL_FILE}",
            file=sys.stderr,
        )
        return 1

    print("knowledge_base schema ready.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(f"ERROR: knowledge base setup failed: {error}", file=sys.stderr)
        sys.exit(1)
