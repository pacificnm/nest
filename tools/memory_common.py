"""Shared helpers for Nest project-memory tools."""

from __future__ import annotations

import os
from collections.abc import Sequence
from pathlib import Path


DEFAULT_DATABASE_URL = "postgresql:///nest_memory?host=/var/run/postgresql"
PROJECT_ROOT = Path(__file__).resolve().parents[1]

_PROVIDER_KEY_VARS = {
    "OPENAI_API_KEY": "openai",
    "ANTHROPIC_API_KEY": "claude",
    "CURSOR_API_KEY": "cursor",
}


def _apply_dotenv_lines(
    lines: list[str],
    *,
    provider_keys_only: bool,
) -> None:
    for raw_line in lines:
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue

        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip().strip('"').strip("'")
        is_provider = key in _PROVIDER_KEY_VARS

        if provider_keys_only:
            if not is_provider or key in os.environ:
                continue
        elif is_provider:
            continue
        elif key in os.environ:
            continue

        if key and value:
            os.environ[key] = value


def load_project_env() -> None:
    """Load repo-local .env values without requiring python-dotenv."""
    env_path = PROJECT_ROOT / ".env"
    if not env_path.is_file():
        return

    _apply_dotenv_lines(
        env_path.read_text(encoding="utf-8").splitlines(),
        provider_keys_only=False,
    )


def load_dotenv_provider_fallback() -> None:
    """Load provider API keys from `.env` only when still unset."""
    env_path = PROJECT_ROOT / ".env"
    if not env_path.is_file():
        return

    _apply_dotenv_lines(
        env_path.read_text(encoding="utf-8").splitlines(),
        provider_keys_only=True,
    )


def load_memory_env() -> None:
    """Load MCP memory env from repo `.env` and provider-key fallback."""
    load_project_env()
    load_dotenv_provider_fallback()


def database_url() -> str:
    load_memory_env()
    return os.environ.get("DATABASE_URL", DEFAULT_DATABASE_URL)


def vector_literal(values: Sequence[float]) -> str:
    """Return a pgvector text literal without requiring the pgvector Python package."""
    return "[" + ",".join(str(float(value)) for value in values) + "]"


load_memory_env()
