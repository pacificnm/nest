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
    """Search indexed reference manuals (Rust book, egui, eframe, etc.)."""
    rows = search_knowledge(query, limit=limit, collection=collection)
    return format_results(query, rows, collection=collection)


@mcp.tool()
def list_knowledge_collections() -> str:
    """List knowledge_base collections that have been indexed."""
    names = list_collections()
    if not names:
        return "No knowledge collections indexed yet. Run ./scripts/index-knowledge.sh"

    return "Indexed collections:\n" + "\n".join(f"- {name}" for name in names)


if __name__ == "__main__":
    mcp.run()
