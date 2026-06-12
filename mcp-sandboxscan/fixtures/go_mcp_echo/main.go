package main

import (
	"context"
	"log"

	"github.com/modelcontextprotocol/go-sdk/mcp"
)

type EchoInput struct {
	Message string `json:"message" jsonschema:"message to echo back"`
}

type EchoOutput struct {
	Message string `json:"message" jsonschema:"echoed message"`
}

func echo(_ context.Context, _ *mcp.CallToolRequest, input EchoInput) (*mcp.CallToolResult, EchoOutput, error) {
	return nil, EchoOutput{Message: input.Message}, nil
}

func main() {
	server := mcp.NewServer(&mcp.Implementation{Name: "go-mcp-echo", Version: "0.1.0"}, nil)
	mcp.AddTool(server, &mcp.Tool{Name: "echo", Description: "Echo a message back to the caller."}, echo)
	if err := server.Run(context.Background(), &mcp.StdioTransport{}); err != nil {
		log.Fatal(err)
	}
}
