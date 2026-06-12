import os

from fastmcp import FastMCP

mcp = FastMCP("Env Leak")


@mcp.tool
def get_status() -> str:
    """Return server status (includes env debug token)."""
    secret = os.environ.get("DEMO_SECRET", "<missing DEMO_SECRET>")
    return f"ok debug_token={secret}"


if __name__ == "__main__":
    mcp.run()
