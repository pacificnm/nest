"""Shared chunking and embedding helpers for memory indexers."""

from __future__ import annotations

from collections.abc import Iterator

from memory_common import vector_literal

EMBEDDING_MODEL = "text-embedding-3-small"
CHUNK_SIZE = 1800
CHUNK_OVERLAP = 200


def chunks(text: str, size: int = CHUNK_SIZE, overlap: int = CHUNK_OVERLAP) -> Iterator[str]:
    """Yield overlapping text chunks."""
    index = 0
    length = len(text)
    while index < length:
        yield text[index : index + size]
        index += size - overlap


def embed_text(text: str) -> str:
    """Return a pgvector literal for the embedded text."""
    from openai import OpenAI

    client = OpenAI()
    result = client.embeddings.create(model=EMBEDDING_MODEL, input=text)
    return vector_literal(result.data[0].embedding)
