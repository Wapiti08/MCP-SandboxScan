from fastmcp import FastMCP

mcp = FastMCP("Echo")


@mcp.tool
def echo(message: str) -> str:
    """Echo a message back to the caller."""
    return message


if __name__ == "__main__":
    mcp.run()
