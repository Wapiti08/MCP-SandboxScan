import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";

const server = new McpServer({
    name: "ts-mcp-echo",
    version: "0.1.0",
});

server.registerTool(
    "echo",
    {
      description: "Echo a message back to the caller.",
      inputSchema: z.object({
        message: z.string(),
      }),
    },
    async ({ message }) => ({
      content: [{ type: "text", text: message }],
    })
);

const transport = new StdioServerTransport();
await server.connect(transport);

