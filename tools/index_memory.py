import hashlib
import sys
from pathlib import Path

from embedding import chunks, embed_text
from memory_common import database_url


PATHS = [
    "README.md",
    "AGENTS.md",
    "docs",
    "tools/MCP-SETUP.md",
    "tools/mcp-memory-setup.md",
]


def app_doc_dirs(project_root: Path):
    # apps/<name>/ are gitignored local product checkouts (see apps/README.md),
    # so they aren't reachable via a fixed PATHS entry or a symlink under docs/
    # (rglob doesn't follow symlinked directories during "**" traversal).
    apps_dir = project_root / "apps"
    if not apps_dir.is_dir():
        return []
    return sorted(p for p in apps_dir.glob("*/docs") if p.is_dir())


def main() -> int:
    import psycopg

    project_root = Path(__file__).resolve().parents[1]

    all_paths = [project_root / root for root in PATHS] + app_doc_dirs(project_root)

    with psycopg.connect(database_url()) as conn:
        for path in all_paths:
            if path.is_dir():
                files = list(path.rglob("*.md"))
            elif path.exists():
                files = [path]
            else:
                continue

            for file in files:
                text = file.read_text(errors="ignore")
                source_path = file.relative_to(project_root).as_posix()

                for chunk in chunks(text):
                    content_hash = hashlib.sha256(
                        f"{source_path}:{chunk}".encode()
                    ).hexdigest()
                    vector = embed_text(chunk)

                    conn.execute(
                        """
                        INSERT INTO project_memory
                          (source_path, content, content_hash, embedding)
                        VALUES
                          (%s, %s, %s, %s::vector)
                        ON CONFLICT (content_hash) DO NOTHING
                        """,
                        (source_path, chunk, content_hash, vector),
                    )

        conn.commit()

    print("Memory indexing complete.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(f"ERROR: memory indexing failed: {error}", file=sys.stderr)
        sys.exit(1)
