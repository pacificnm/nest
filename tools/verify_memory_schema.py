"""Verify nest_memory has all required memory tables."""

from __future__ import annotations

import sys

from memory_common import database_url

REQUIRED_TABLES = (
    "project_memory",
    "agent_context_memory",
    "knowledge_base",
)


def main() -> int:
    import psycopg

    try:
        with psycopg.connect(database_url()) as conn:
            rows = conn.execute(
                """
                SELECT table_name
                FROM information_schema.tables
                WHERE table_schema = 'public'
                  AND table_type = 'BASE TABLE'
                ORDER BY table_name
                """
            ).fetchall()

            present = {row[0] for row in rows}
            missing = [name for name in REQUIRED_TABLES if name not in present]

            print(f"Database: {database_url().split('@')[-1]}")
            print("Tables in public schema:")
            for name in sorted(present):
                marker = "ok" if name in REQUIRED_TABLES else "extra"
                print(f"  - {name} ({marker})")

            if missing:
                print("\nMISSING required tables:", ", ".join(missing), file=sys.stderr)
                print(
                    "Fix by running the full schema setup:\n"
                    "  ./scripts/setup-database-postgres.sh\n"
                    "Or context memory only:\n"
                    "  ./scripts/setup-context-memory.sh",
                    file=sys.stderr,
                )
                return 1

            print("\nAll required tables present.")
            for table in REQUIRED_TABLES:
                count = conn.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0]
                print(f"  {table}: {count} rows")

            return 0
    except Exception as error:
        print(f"ERROR: cannot connect or verify database: {error}", file=sys.stderr)
        print(
            "Check DATABASE_URL in .env and create the database:\n"
            "  sudo -u postgres createdb nest_memory\n"
            "  ./scripts/setup-database-postgres.sh",
            file=sys.stderr,
        )
        return 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(f"ERROR: schema verification failed: {error}", file=sys.stderr)
        sys.exit(1)
