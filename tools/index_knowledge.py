"""Index external reference manuals into knowledge_base."""

from __future__ import annotations

import hashlib
import sys
from pathlib import Path

from embedding import chunks, embed_text
from memory_common import PROJECT_ROOT, database_url


def load_collections(config_path: Path) -> list[dict]:
    """Load [[collections]] entries from a TOML config file."""
    text = config_path.read_text(encoding="utf-8")
    try:
        import tomllib

        data = tomllib.loads(text)
    except ImportError:
        import toml  # type: ignore[import-untyped]

        data = toml.loads(text)

    collections = data.get("collections", [])
    if not isinstance(collections, list):
        raise ValueError("config must contain a [[collections]] array")

    parsed: list[dict] = []
    for entry in collections:
        if not isinstance(entry, dict):
            continue
        name = str(entry.get("name", "")).strip()
        source = str(entry.get("source", "")).strip()
        extensions = entry.get("extensions", [])
        if not name or not source:
            raise ValueError("each collection needs name and source")
        if not isinstance(extensions, list):
            extensions = []
        parsed.append(
            {
                "name": name,
                "source": Path(source).expanduser(),
                "extensions": [str(ext).lstrip(".").lower() for ext in extensions],
            }
        )
    return parsed


def iter_collection_files(source: Path, extensions: list[str]) -> list[Path]:
    """Return files under source matching extensions (empty = all UTF-8 text files)."""
    if not source.is_dir():
        return []

    files: list[Path] = []
    for path in source.rglob("*"):
        if not path.is_file():
            continue
        if extensions:
            ext = path.suffix.lstrip(".").lower()
            if ext not in extensions:
                continue
        files.append(path)
    return files


def index_collection(
    conn,
    *,
    collection: str,
    source: Path,
    extensions: list[str],
) -> tuple[int, int]:
    """Index one collection. Returns (chunks_indexed, files_seen)."""
    files = iter_collection_files(source, extensions)
    indexed = 0

    for file in files:
        try:
            text = file.read_text(encoding="utf-8", errors="ignore").strip()
        except OSError:
            continue
        if not text:
            continue

        rel = file.relative_to(source).as_posix()
        for chunk in chunks(text):
            content_hash = hashlib.sha256(
                f"{collection}:{rel}:{chunk}".encode()
            ).hexdigest()
            vector = embed_text(chunk)
            conn.execute(
                """
                INSERT INTO knowledge_base
                  (collection, source_path, content, content_hash, embedding)
                VALUES
                  (%s, %s, %s, %s, %s::vector)
                ON CONFLICT (content_hash) DO NOTHING
                """,
                (collection, rel, chunk, content_hash, vector),
            )
            indexed += 1

    return indexed, len(files)


def main() -> int:
    import argparse
    import psycopg

    parser = argparse.ArgumentParser(description="Index reference manuals into knowledge_base")
    parser.add_argument(
        "--config",
        type=Path,
        default=PROJECT_ROOT / "tools" / "knowledge.toml",
        help="TOML file with [[collections]] entries",
    )
    args = parser.parse_args()

    if not args.config.is_file():
        print(
            f"ERROR: config not found: {args.config}\n"
            "Copy tools/knowledge.toml.example to tools/knowledge.toml or run ./scripts/index-knowledge.sh",
            file=sys.stderr,
        )
        return 1

    collections = load_collections(args.config)
    if not collections:
        print("ERROR: no collections defined in config", file=sys.stderr)
        return 1

    with psycopg.connect(database_url()) as conn:
        for entry in collections:
            source = entry["source"]
            if not source.is_dir():
                print(f"WARNING: skipping {entry['name']}: missing {source}", file=sys.stderr)
                continue
            print(f"Indexing {entry['name']} from {source}...")
            indexed, files_seen = index_collection(
                conn,
                collection=entry["name"],
                source=source,
                extensions=entry["extensions"],
            )
            print(f"  {files_seen} files, {indexed} chunks")
        conn.commit()

    print("Knowledge indexing complete.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(f"ERROR: knowledge indexing failed: {error}", file=sys.stderr)
        sys.exit(1)
