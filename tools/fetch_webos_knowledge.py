"""Fetch LG webOS TV developer docs into markdown for nest-knowledge indexing."""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

import httpx
from bs4 import BeautifulSoup
from markdownify import markdownify as html_to_md

from memory_common import PROJECT_ROOT

USER_AGENT = "LoonDocMirror/1.0 (+https://github.com/pacificnm/loon; dev documentation cache)"
REQUEST_DELAY_SEC = 1.0


def load_pages(config_path: Path) -> list[dict]:
    """Load [[pages]] entries from a TOML config file."""
    text = config_path.read_text(encoding="utf-8")
    try:
        import tomllib

        data = tomllib.loads(text)
    except ImportError:
        import toml  # type: ignore[import-untyped]

        data = toml.loads(text)

    pages = data.get("pages", [])
    if not isinstance(pages, list):
        raise ValueError("config must contain a [[pages]] array")

    parsed: list[dict] = []
    for entry in pages:
        if not isinstance(entry, dict):
            continue
        slug = str(entry.get("slug", "")).strip()
        url = str(entry.get("url", "")).strip()
        if not slug or not url:
            raise ValueError("each page needs slug and url")
        parsed.append({"slug": slug, "url": url})
    return parsed


def extract_main_html(soup: BeautifulSoup) -> BeautifulSoup:
    """Pick the main documentation content, skipping chrome."""
    for selector in ("article", "main", '[role="main"]', ".content", "#content"):
        node = soup.select_one(selector)
        if node and len(node.get_text(strip=True)) > 200:
            return node

    # LG docs: first h1 and its following siblings in body
    h1 = soup.find("h1")
    if h1:
        wrapper = soup.new_tag("div")
        for sibling in h1.find_all_next():
            if sibling.name in {"footer", "nav"}:
                break
            if getattr(sibling, "name", None) == "h1" and sibling is not h1:
                break
            wrapper.append(sibling.extract() if sibling.parent else sibling)
        h1.extract()
        wrapper.insert(0, h1)
        if len(wrapper.get_text(strip=True)) > 100:
            return wrapper

    body = soup.body or soup
    for tag in body.find_all(["script", "style", "nav", "header", "footer", "noscript"]):
        tag.decompose()
    return body


def clean_markdown(text: str) -> str:
    """Normalize whitespace and drop site chrome lines."""
    lines: list[str] = []
    skip_patterns = (
        re.compile(r"^Site Logo$"),
        re.compile(r"^Sign In$"),
        re.compile(r"^Search$"),
        re.compile(r"^LG Electronics Logo$"),
        re.compile(r"^Copyright ©"),
        re.compile(r"^\[TERMS"),
        re.compile(r"^No Headings$"),
        re.compile(r"^Open menu$"),
        re.compile(r"^Close menu$"),
    )
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped:
            if lines and lines[-1] != "":
                lines.append("")
            continue
        if any(p.match(stripped) for p in skip_patterns):
            continue
        lines.append(line.rstrip())

    return "\n".join(lines).strip() + "\n"


def fetch_page(client: httpx.Client, url: str) -> tuple[str, str]:
    """Return (title, markdown body) for a documentation URL."""
    response = client.get(url, follow_redirects=True)
    response.raise_for_status()

    soup = BeautifulSoup(response.text, "html.parser")
    title_tag = soup.find("h1")
    title = title_tag.get_text(strip=True) if title_tag else url.rsplit("/", 1)[-1]

    content = extract_main_html(soup)
    md = html_to_md(str(content), heading_style="ATX", bullets="-")
    md = clean_markdown(md)
    return title, md


def write_page(
    output_dir: Path,
    *,
    slug: str,
    url: str,
    title: str,
    body: str,
) -> Path:
    """Write one markdown file with YAML frontmatter."""
    path = output_dir / f"{slug}.md"
    path.parent.mkdir(parents=True, exist_ok=True)
    fetched_at = datetime.now(timezone.utc).isoformat()
    frontmatter = (
        f"---\n"
        f"source_url: {url}\n"
        f"title: {title}\n"
        f"fetched_at: {fetched_at}\n"
        f"---\n\n"
    )
    # Drop duplicate leading h1 from converted HTML.
    body_stripped = re.sub(rf"^#\s+{re.escape(title)}\s*\n+", "", body, count=1, flags=re.IGNORECASE)
    path.write_text(frontmatter + f"# {title}\n\n{body_stripped}", encoding="utf-8")
    return path


def main() -> int:
    parser = argparse.ArgumentParser(description="Fetch webOS TV docs as markdown")
    parser.add_argument(
        "--config",
        type=Path,
        default=PROJECT_ROOT / "tools" / "webos-knowledge-urls.toml",
        help="TOML file with [[pages]] entries",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Output directory (default: $NEST_KNOWLEDGE/webos-tv)",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Re-fetch even if manifest already has this URL",
    )
    args = parser.parse_args()

    import os

    knowledge_root = Path(os.environ.get("NEST_KNOWLEDGE", "")).expanduser()
    if not knowledge_root:
        for candidate in (
            Path("/data/nest-knowledge"),
            PROJECT_ROOT / "data" / "nest-knowledge",
        ):
            if candidate.is_dir() and os.access(candidate, os.W_OK):
                knowledge_root = candidate
                break
        else:
            knowledge_root = PROJECT_ROOT / "data" / "nest-knowledge"
    output_dir = (args.output or knowledge_root / "webos-tv").resolve()
    manifest_path = output_dir / "manifest.json"

    if not args.config.is_file():
        print(f"ERROR: config not found: {args.config}", file=sys.stderr)
        return 1

    pages = load_pages(args.config)
    if not pages:
        print("ERROR: no pages in config", file=sys.stderr)
        return 1

    manifest: dict[str, dict] = {}
    if manifest_path.is_file():
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

    output_dir.mkdir(parents=True, exist_ok=True)
    headers = {"User-Agent": USER_AGENT, "Accept": "text/html,application/xhtml+xml"}

    ok = 0
    with httpx.Client(headers=headers, timeout=60.0) as client:
        for index, page in enumerate(pages):
            slug = page["slug"]
            url = page["url"]
            if not args.force and manifest.get(slug, {}).get("url") == url:
                existing = output_dir / f"{slug}.md"
                if existing.is_file():
                    print(f"SKIP {slug} (unchanged)")
                    ok += 1
                    continue

            print(f"FETCH {url}")
            try:
                title, body = fetch_page(client, url)
                path = write_page(output_dir, slug=slug, url=url, title=title, body=body)
                manifest[slug] = {
                    "url": url,
                    "title": title,
                    "path": path.relative_to(output_dir).as_posix(),
                    "fetched_at": datetime.now(timezone.utc).isoformat(),
                    "bytes": path.stat().st_size,
                }
                ok += 1
                print(f"  -> {path} ({path.stat().st_size} bytes)")
            except Exception as error:
                print(f"  ERROR: {error}", file=sys.stderr)

            if index + 1 < len(pages):
                time.sleep(REQUEST_DELAY_SEC)

    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"\nWrote {ok}/{len(pages)} pages to {output_dir}")
    print(f"Manifest: {manifest_path}")

    if ok < len(pages):
        return 1
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(f"ERROR: {error}", file=sys.stderr)
        sys.exit(1)
