"""Shared chunking and embedding helpers for memory indexers."""

from __future__ import annotations

import os
from collections.abc import Iterator

import httpx

from memory_common import vector_literal

OLLAMA_HOST = os.environ.get("OLLAMA_HOST", "http://127.0.0.1:11434")
EMBEDDING_MODEL = os.environ.get("OLLAMA_EMBED_MODEL", "nomic-embed-text")
CHUNK_SIZE = 1800
CHUNK_OVERLAP = 200

# nomic-embed-text caps input at 2048 tokens, and tool-call JSON tokenizes
# less efficiently than prose, so a single call (e.g. a pre-compaction
# context-memory checkpoint spanning dozens of messages) can exceed that
# well before it looks large in characters. Embed only the tail (most
# recent, most relevant content) past this size; callers still store the
# untruncated text separately.
MAX_EMBED_CHARS = 6000


def chunks(text: str, size: int = CHUNK_SIZE, overlap: int = CHUNK_OVERLAP) -> Iterator[str]:
    """Yield overlapping text chunks."""
    index = 0
    length = len(text)
    while index < length:
        yield text[index : index + size]
        index += size - overlap


def embed_text(text: str) -> str:
    """Return a pgvector literal for the embedded text."""
    if len(text) > MAX_EMBED_CHARS:
        text = text[-MAX_EMBED_CHARS:]

    response = httpx.post(
        f"{OLLAMA_HOST}/api/embed",
        json={"model": EMBEDDING_MODEL, "input": text},
        timeout=30.0,
    )
    response.raise_for_status()
    embedding = response.json()["embeddings"][0]
    return vector_literal(embedding)
