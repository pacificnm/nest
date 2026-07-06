try:
    from mcp.server.fastmcp import FastMCP
except ModuleNotFoundError as error:
    raise SystemExit(
        "Missing Python dependency for Nest knowledge MCP. "
        "Run it with .venv/bin/python or install the project Python dependencies."
    ) from error

from search_knowledge import format_results, list_collections, search_knowledge


mcp = FastMCP("nest-knowledge")


@mcp.tool()
def search_knowledge_base(query: str, limit: int = 8, collection: str = "") -> str:
    """Search indexed reference manuals (Rust book, Tauri, React, Tailwind, etc.)."""
    rows = search_knowledge(query, limit=limit, collection=collection)
    return format_results(query, rows, collection=collection)


@mcp.tool()
def list_knowledge_collections() -> str:
    """List knowledge_base collections that have been indexed."""
    names = list_collections()
    if not names:
        return (
            "No reference-manual collections indexed in knowledge_base yet.\n"
            "This is separate from nest-memory (project docs in PostgreSQL).\n"
            "Run: NEST_KNOWLEDGE=~/nest-knowledge ./scripts/index-knowledge.sh\n"
            "For Nest project docs, use nest-memory search_project_memory instead."
        )

    return "Indexed collections:\n" + "\n".join(f"- {name}" for name in names)


if __name__ == "__main__":
    mcp.run()
