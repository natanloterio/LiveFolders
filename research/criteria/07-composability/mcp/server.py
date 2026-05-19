from mcp.server.fastmcp import FastMCP

mcp = FastMCP("pipeline")

def _shout(text: str) -> str:
    return text.upper()

def _word_count(text: str) -> int:
    return len(text.split())

@mcp.tool()
def shout_and_count(text: str) -> str:
    """Shouts input then counts words."""
    shouted = _shout(text)
    words = _word_count(shouted)
    return f"Text: {shouted} | Words: {words}"

if __name__ == "__main__":
    mcp.run()
