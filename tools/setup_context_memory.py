"""Create the agent_context_memory table and indexes.

Prefer ./scripts/setup-database-postgres.sh for full schema setup.
"""

import sys

from memory_common import database_url
from setup_database import CONTEXT_SCHEMA_SQL


def main() -> int:
    import psycopg

    try:
        with psycopg.connect(database_url()) as conn:
            conn.execute(CONTEXT_SCHEMA_SQL)
            conn.commit()
    except psycopg.errors.InsufficientPrivilege:
        print(
            "ERROR: current database user cannot create tables in schema public.\n"
            "Run:\n"
            "  ./scripts/setup-context-memory.sh\n"
            "Or full schema:\n"
            f"  ./scripts/setup-database-postgres.sh",
            file=sys.stderr,
        )
        return 1

    print("agent_context_memory schema ready.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(f"ERROR: context memory setup failed: {error}", file=sys.stderr)
        sys.exit(1)
