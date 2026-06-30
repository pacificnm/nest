"""Per-conversation memory gate state for Cursor hooks."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from memory_common import PROJECT_ROOT

STATE_PATH = PROJECT_ROOT / ".cursor" / "memory-gate-state.json"

MEMORY_TOOL_MARKERS = (
    "search_project_memory",
    "search_context_memory",
    "list_context_memory",
    "get_context_memory",
    "save_context_memory",
    "search_knowledge_base",
    "list_knowledge_collections",
)

MEMORY_SERVER_MARKERS = (
    "nest-memory",
    "nest-context-memory",
    "nest-knowledge",
)


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def _load() -> dict[str, Any]:
    if not STATE_PATH.is_file():
        return {"conversations": {}}
    try:
        data = json.loads(STATE_PATH.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {"conversations": {}}
    if not isinstance(data, dict):
        return {"conversations": {}}
    data.setdefault("conversations", {})
    return data


def _save(data: dict[str, Any]) -> None:
    STATE_PATH.parent.mkdir(parents=True, exist_ok=True)
    STATE_PATH.write_text(json.dumps(data, indent=2), encoding="utf-8")


def conversation_id(payload: dict) -> str:
    for key in ("conversation_id", "session_id", "chat_id"):
        value = str(payload.get(key, "")).strip()
        if value:
            return value
    return "default"


def get_state(payload: dict) -> dict[str, Any]:
    data = _load()
    conv_id = conversation_id(payload)
    conversations = data["conversations"]
    if conv_id not in conversations:
        conversations[conv_id] = {
            "session_key": "",
            "project_memory_ok": False,
            "context_read_ok": False,
            "saved_this_turn": False,
            "last_save_at": None,
            "updated_at": _now_iso(),
        }
        _save(data)
    return conversations[conv_id]


def reset_session(payload: dict, *, session_key: str = "") -> None:
    data = _load()
    conv_id = conversation_id(payload)
    data["conversations"][conv_id] = {
        "session_key": session_key,
        "project_memory_ok": False,
        "context_read_ok": False,
        "saved_this_turn": False,
        "last_save_at": None,
        "updated_at": _now_iso(),
    }
    _save(data)


def update_state(payload: dict, **fields: Any) -> dict[str, Any]:
    data = _load()
    conv_id = conversation_id(payload)
    state = get_state(payload)
    state.update(fields)
    state["updated_at"] = _now_iso()
    data["conversations"][conv_id] = state
    _save(data)
    return state


def gate_open(state: dict[str, Any]) -> bool:
    return bool(state.get("project_memory_ok") and state.get("context_read_ok"))


def is_memory_tool(payload: dict) -> bool:
    blob = json.dumps(payload).lower()
    if any(marker in blob for marker in MEMORY_TOOL_MARKERS):
        return True
    return any(marker in blob for marker in MEMORY_SERVER_MARKERS)


def tool_name(payload: dict) -> str:
    for key in ("tool_name", "tool", "name", "toolName"):
        value = payload.get(key)
        if isinstance(value, str) and value.strip():
            return value.strip()
    return ""


def mark_from_tool(payload: dict) -> None:
    """Update gate flags based on a completed memory MCP tool call."""
    blob = json.dumps(payload).lower()
    fields: dict[str, Any] = {}

    if "search_project_memory" in blob:
        fields["project_memory_ok"] = True
    if "search_context_memory" in blob or "list_context_memory" in blob:
        fields["context_read_ok"] = True
    if "save_context_memory" in blob:
        fields["saved_this_turn"] = True
        fields["last_save_at"] = _now_iso()

    if fields:
        update_state(payload, **fields)


def begin_agent_turn(payload: dict) -> None:
    """Next agent turn requires a fresh save_context_memory call."""
    update_state(payload, saved_this_turn=False)
