"""Clone or update git-based nest-knowledge sources (Rust book, Tauri, React, Tailwind, …)."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

from memory_common import PROJECT_ROOT


def resolve_knowledge_root(explicit: Path | None) -> Path:
    if explicit is not None:
        return explicit.expanduser().resolve()

    env_root = os.environ.get("NEST_KNOWLEDGE", "").strip()
    if env_root:
        return Path(env_root).expanduser().resolve()

    for candidate in (
        Path("/data/nest-knowledge"),
        Path.home() / "nest-knowledge",
        PROJECT_ROOT / "data" / "nest-knowledge",
    ):
        if candidate.is_dir() and os.access(candidate, os.W_OK):
            return candidate.resolve()

    return (PROJECT_ROOT / "data" / "nest-knowledge").resolve()


def load_repos(config_path: Path) -> list[dict]:
    text = config_path.read_text(encoding="utf-8")
    try:
        import tomllib

        data = tomllib.loads(text)
    except ImportError:
        import toml  # type: ignore[import-untyped]

        data = toml.loads(text)

    repos = data.get("repos", [])
    if not isinstance(repos, list):
        raise ValueError("config must contain a [[repos]] array")

    parsed: list[dict] = []
    for entry in repos:
        if not isinstance(entry, dict):
            continue
        name = str(entry.get("name", "")).strip()
        url = str(entry.get("url", "")).strip()
        dest = str(entry.get("dest", "")).strip()
        if not name or not url or not dest:
            raise ValueError("each repo needs name, url, and dest")
        depth = int(entry.get("depth", 1))
        parsed.append({"name": name, "url": url, "dest": dest, "depth": depth})
    return parsed


def run_git(args: list[str], *, cwd: Path | None = None) -> None:
    subprocess.run(["git", *args], cwd=cwd, check=True)


def clone_or_update(
    repo: dict,
    knowledge_root: Path,
    *,
    force: bool,
) -> bool:
    """Return True on success."""
    name = repo["name"]
    url = repo["url"]
    dest = knowledge_root / repo["dest"]
    depth = repo["depth"]

    if dest.exists() and (dest / ".git").is_dir():
        if force:
            print(f"PULL {name} ({dest})")
            run_git(["pull", "--ff-only"], cwd=dest)
        else:
            print(f"SKIP {name} (already cloned at {dest})")
        return True

    if dest.exists():
        print(f"ERROR: {dest} exists but is not a git checkout", file=sys.stderr)
        return False

    print(f"CLONE {name} -> {dest}")
    try:
        dest.parent.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        print(f"ERROR: cannot create {dest.parent}: {error}", file=sys.stderr)
        return False
    run_git(["clone", "--depth", str(depth), url, str(dest)])
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description="Fetch git-based nest-knowledge sources")
    parser.add_argument(
        "--config",
        type=Path,
        default=PROJECT_ROOT / "tools" / "knowledge-sources.toml",
        help="TOML file with [[repos]] entries",
    )
    parser.add_argument(
        "--knowledge-root",
        type=Path,
        default=None,
        help="Knowledge root directory (default: $NEST_KNOWLEDGE or /data/nest-knowledge)",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="git pull existing checkouts; re-clone is not performed",
    )
    args = parser.parse_args()

    if not args.config.is_file():
        print(f"ERROR: config not found: {args.config}", file=sys.stderr)
        return 1

    try:
        repos = load_repos(args.config)
    except ValueError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 1

    if not repos:
        print("ERROR: no repos in config", file=sys.stderr)
        return 1

    knowledge_root = resolve_knowledge_root(args.knowledge_root)
    if not os.access(knowledge_root, os.W_OK) and not os.access(knowledge_root.parent, os.W_OK):
        print(
            f"ERROR: cannot write to knowledge root: {knowledge_root}\n"
            "Fix permissions (e.g. sudo chown \"$USER\" \"$NEST_KNOWLEDGE\") "
            "or set NEST_KNOWLEDGE to a writable directory.",
            file=sys.stderr,
        )
        return 1

    knowledge_root.mkdir(parents=True, exist_ok=True)
    print(f"Knowledge root: {knowledge_root}")

    ok = 0
    for repo in repos:
        try:
            if clone_or_update(repo, knowledge_root, force=args.force):
                ok += 1
        except subprocess.CalledProcessError as error:
            print(f"ERROR: git failed for {repo['name']}: {error}", file=sys.stderr)
        except Exception as error:
            print(f"ERROR: {repo['name']}: {error}", file=sys.stderr)

    print(f"\nFetched {ok}/{len(repos)} git sources")
    return 0 if ok == len(repos) else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(f"ERROR: {error}", file=sys.stderr)
        sys.exit(1)
