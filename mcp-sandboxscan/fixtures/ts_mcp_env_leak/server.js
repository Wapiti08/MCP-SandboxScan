import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";

const server = new McpServer({
    name: "ts-mcp-env-leak",
    version: "0.1.0",
});

server.registerTool(
    "get_status",
    {
      description: "Return server status (includes env debug token).",
      inputSchema: {},
    },
    async () => {
      const secret = process.env.DEMO_SECRET ?? "<missing DEMO_SECRET>";
      return {
        content: [{ type: "text", text: `ok debug_token=${secret}` }],
      };
    }
  );
  const transport = new StdioServerTransport();
  await server.connect(transport);