"""Cursor hook handlers for Nest mandatory memory workflow."""

from __future__ import annotations

import json
import sys

from context_memory import format_entry, list_context
from memory_gate import (
    begin_agent_turn,
    conversation_id,
    gate_open,
    get_state,
    is_memory_tool,
    is_webos_work,
    mark_from_tool,
    reset_session,
    tool_name,
    update_state,
)
from transcript_snapshot import git_branch, snapshot_transcript

SESSION_KEY_HINT = "{session_key}"

MEMORY_REQUIREMENTS = f"""## Nest memory requirements (MANDATORY — enforced by hooks)

You MUST use MCP memory tools in this order before editing code, running shell
commands, or making implementation changes:

1. **`search_project_memory`** (`nest-memory`) — Nest plans, crate docs, decisions.
2. **`search_context_memory`** OR **`list_context_memory`** (`nest-context-memory`)
   with `session_key="{SESSION_KEY_HINT}"` — resume prior work for this branch/session.

Optional when using Rust/egui APIs: **`search_knowledge_base`** (`nest-knowledge`).

### webOS TV client (`apps/loon/client/`)

When editing the Loon webOS client, hooks also require **`search_knowledge_base`**
with `collection="webos-tv"` before other tools. See `.cursor/rules/webos-tv-knowledge.mdc`.

### Save context (required)

- **After every agent response** that changes understanding or code: call
  **`save_context_memory`** with decisions, files touched, blockers, and verification.
- **Before context compaction**: call **`save_context_memory`** with a full checkpoint.
  A pre-compaction hook also snapshots the transcript automatically.

Use `session_key="{SESSION_KEY_HINT}"` on all context memory calls.

Hooks block non-memory tools until steps 1–2 are complete (and `webos-tv` knowledge
when editing `apps/loon/client/`).
"""


GATE_DENY_MESSAGE = """Nest memory gate is closed.

Before editing files or running commands you MUST:
1. Call MCP `search_project_memory` (nest-memory) for this task.
2. Call MCP `search_context_memory` or `list_context_memory` (nest-context-memory)
   with session_key="{session_key}".

Then retry your action."""


SAVE_FOLLOWUP_MESSAGE = """Required before ending this turn: call MCP `save_context_memory`
(nest-context-memory) with a concise checkpoint:

- What you did this turn
- Files changed or inspected
- Decisions and blockers
- Commands run and results

Use session_key="{session_key}" and a short title."""


WEBOS_GATE_DENY_MESSAGE = """webOS TV knowledge gate is closed.

You are working on the Loon webOS client. After project + context memory, call:

  search_knowledge_base(query="<your platform topic>", collection="webos-tv")

Examples: appinfo.json fields, disableBackHistoryAPI, webOSRelaunch, ares-install.

Then retry your action."""


def read_hook_input() -> dict:
    raw = sys.stdin.read()
    if not raw.strip():
        return {}
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError:
        return {}
    return payload if isinstance(payload, dict) else {}


def resolve_session_key(payload: dict) -> str:
    state = get_state(payload)
    if state.get("session_key"):
        return str(state["session_key"])

    branch = git_branch()
    conv_id = conversation_id(payload)
    if branch and conv_id != "default":
        return f"{branch}:{conv_id[:8]}"
    return branch or conv_id


def recent_context_summary(session_key: str, *, limit: int = 3) -> str:
    if not session_key:
        return ""

    try:
        branch = git_branch()
        rows = list_context(limit=limit, session_key=session_key)
        if not rows and branch and branch != session_key:
            rows = list_context(limit=limit, session_key=branch)
    except Exception:
        return ""

    if not rows:
        return (
            "## Recent context memory\n\n"
            "No saved context yet for this session. After reading project memory, "
            "check again with `list_context_memory`."
        )

    blocks = [format_entry(*row, content_limit=800) for row in rows]
    return "## Recent context memory\n\n" + "\n\n".join(blocks)


def session_start() -> int:
    payload = read_hook_input()
    session_key = resolve_session_key(payload)
    reset_session(payload, session_key=session_key)

    context = MEMORY_REQUIREMENTS.replace(SESSION_KEY_HINT, session_key or "(git branch)")
    recent = recent_context_summary(session_key)
    context = f"{context}\n\n{recent}"

    print(json.dumps({"additional_context": context}))
    return 0


def pre_tool_use() -> int:
    payload = read_hook_input()
    session_key = resolve_session_key(payload)
    state = get_state(payload)

    if is_memory_tool(payload):
        print(json.dumps({"permission": "allow"}))
        return 0

    if is_webos_work(payload) and not state.get("webos_context_active"):
        state = update_state(payload, webos_context_active=True)

    if gate_open(state):
        print(json.dumps({"permission": "allow"}))
        return 0

    name = tool_name(payload)
    if state.get("webos_context_active") and not state.get("webos_knowledge_ok"):
        agent_message = WEBOS_GATE_DENY_MESSAGE
        user_message = (
            "webOS knowledge gate blocked a tool until search_knowledge_base "
            f"(collection=webos-tv) runs. Blocked tool: {name or 'unknown'}."
        )
    else:
        agent_message = GATE_DENY_MESSAGE.format(session_key=session_key or "(git branch)")
        user_message = (
            "Memory gate blocked a tool until project and context memory are queried. "
            f"Blocked tool: {name or 'unknown'}."
        )
    print(
        json.dumps(
            {
                "permission": "deny",
                "agent_message": agent_message,
                "user_message": user_message,
            }
        )
    )
    return 0


def post_tool_use() -> int:
    payload = read_hook_input()
    mark_from_tool(payload)
    print(json.dumps({}))
    return 0


def after_mcp_execution() -> int:
    payload = read_hook_input()
    mark_from_tool(payload)
    print(json.dumps({}))
    return 0


def pre_compact() -> int:
    import os

    payload = read_hook_input()
    session_key = resolve_session_key(payload)
    transcript_path = (
        str(payload.get("transcript_path", "")).strip()
        or os.environ.get("CURSOR_TRANSCRIPT_PATH", "").strip()
    )

    trigger = str(payload.get("trigger", "auto"))
    conv_id = conversation_id(payload)
    tags = ["pre-compact", "auto"] + ([conv_id[:8]] if conv_id != "default" else [])

    user_message = (
        f"Compaction ({trigger}) starting. Required: call `save_context_memory` with "
        f"session_key=\"{session_key}\" if you have not saved a checkpoint this turn."
    )

    if transcript_path:
        try:
            entry_id = snapshot_transcript(
                transcript_path,
                session_key=session_key,
                title=f"Pre-compaction snapshot ({trigger})",
                tags=tags,
            )
            user_message = (
                f"Saved automatic pre-compaction snapshot (entry id={entry_id}). "
                f"Also call `save_context_memory` with session_key=\"{session_key}\" "
                "summarizing decisions and file paths before continuing after compaction."
            )
        except Exception as error:
            user_message = (
                "Compaction starting; automatic snapshot failed "
                f"({error}). You MUST call `save_context_memory` manually before continuing."
            )

    begin_agent_turn(payload)
    print(json.dumps({"user_message": user_message}))
    return 0


def stop() -> int:
    payload = read_hook_input()
    session_key = resolve_session_key(payload)
    state = get_state(payload)

    if state.get("saved_this_turn"):
        print(json.dumps({}))
        return 0

    followup = SAVE_FOLLOWUP_MESSAGE.format(session_key=session_key or "(git branch)")
    print(json.dumps({"followup_message": followup}))
    return 0


def main() -> int:
    if len(sys.argv) != 2:
        print(
            "Usage: .venv/bin/python tools/memory_hooks.py "
            "<session-start|pre-tool-use|post-tool-use|after-mcp-execution|pre-compact|stop>",
            file=sys.stderr,
        )
        return 1

    handlers = {
        "session-start": session_start,
        "pre-tool-use": pre_tool_use,
        "post-tool-use": post_tool_use,
        "after-mcp-execution": after_mcp_execution,
        "pre-compact": pre_compact,
        "stop": stop,
    }
    command = sys.argv[1]
    handler = handlers.get(command)
    if handler is None:
        print(f"Unknown hook command: {command}", file=sys.stderr)
        return 1
    return handler()


if __name__ == "__main__":
    raise SystemExit(main())
