#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from context_memory import list_context, save_context


def normalize_text(value: Any) -> str:
    if isinstance(value, str):
        return value.strip()

    if isinstance(value, list):
        parts: list[str] = []

        for item in value:
            if isinstance(item, str):
                parts.append(item)
                continue

            if not isinstance(item, dict):
                continue

            item_type = str(item.get("type", "")).strip()

            if item_type in {"text", "reasoning"}:
                text = item.get("text")
                if isinstance(text, str) and text.strip():
                    parts.append(text.strip())

            elif item_type in {"tool", "tool-call"}:
                tool_name = (
                    item.get("tool")
                    or item.get("toolName")
                    or item.get("name")
                    or "unknown"
                )
                state = item.get("state", {})
                parts.append(
                    f"[tool {tool_name}] "
                    f"{json.dumps(state, ensure_ascii=False, default=str)}"
                )

        return "\n".join(parts).strip()

    if isinstance(value, dict):
        for key in ("text", "content", "message"):
            if key in value:
                text = normalize_text(value[key])
                if text:
                    return text

    return ""


def format_messages(messages: list[dict[str, Any]], limit: int) -> str:
    blocks: list[str] = []

    for message in messages[-limit:]:
        info = message.get("info", message)
        role = str(info.get("role", "unknown"))

        parts = message.get("parts")
        if parts is None:
            parts = info.get("content", "")

        text = normalize_text(parts)
        if not text:
            continue

        blocks.append(f"## {role}\n{text}")

    return "\n\n".join(blocks).strip()


def recent_memory(session_key: str, limit: int = 3) -> str:
    rows = list_context(limit=limit, session_key=session_key)

    if not rows:
        return "No earlier context-memory entries exist for this session."

    blocks: list[str] = []

    for entry_id, key, title, content, tags, created_at in rows:
        blocks.append(
            "\n".join(
                [
                    f"Entry ID: {entry_id}",
                    f"Created: {created_at}",
                    f"Session: {key}",
                    f"Title: {title or '(untitled)'}",
                    f"Tags: {', '.join(tags) if tags else '(none)'}",
                    content,
                ]
            )
        )

    return "\n\n---\n\n".join(blocks)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--session-key", required=True)
    parser.add_argument("--branch", default="")
    parser.add_argument("--repository", default="")
    parser.add_argument("--message-limit", type=int, default=40)
    parser.add_argument("--title", default="OpenCode pre-compaction checkpoint")
    args = parser.parse_args()

    payload = json.load(sys.stdin)
    messages = payload.get("messages", [])

    if not isinstance(messages, list):
        raise ValueError("messages must be a list")

    transcript = format_messages(messages, args.message_limit)

    content = "\n".join(
        [
            "# OpenCode pre-compaction checkpoint",
            "",
            f"Repository: {args.repository or '(unknown)'}",
            f"Branch: {args.branch or '(unknown)'}",
            f"Session key: {args.session_key}",
            "",
            "## Recent conversation",
            "",
            transcript or "No usable messages were found.",
        ]
    )

    entry_id = save_context(
        content,
        title=args.title,
        session_key=args.session_key,
        tags=["opencode", "pre-compact", args.branch or "no-branch"],
    )

    result = {
        "entry_id": entry_id,
        "session_key": args.session_key,
        "context": recent_memory(args.session_key),
    }

    json.dump(result, sys.stdout)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        json.dump({"error": str(error)}, sys.stdout)
        sys.stdout.write("\n")
        sys.exit(1)