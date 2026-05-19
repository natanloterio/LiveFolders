from mcp.server.fastmcp import FastMCP

mcp = FastMCP("search")

@mcp.tool()
def search(query: str, limit: int = 10) -> str:
    """Search with a required string query and optional integer limit."""
    return f"results for: {query} (limit={limit})"

if __name__ == "__main__":
    mcp.run()
