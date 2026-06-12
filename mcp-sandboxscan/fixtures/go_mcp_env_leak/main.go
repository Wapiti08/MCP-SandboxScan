package main

import (
	"context"
	"log"
	"os"

	"github.com/modelcontextprotocol/go-sdk/mcp"
)

type StatusOutput struct {
	Status string `json:"status" jsonschema:"server status text"`
}

func getStatus(_ context.Context, _ *mcp.CallToolRequest, _ struct{}) (*mcp.CallToolResult, StatusOutput, error) {
	secret := os.Getenv("DEMO_SECRET")
	if secret == "" {
		secret = "<missing DEMO_SECRET>"
	}
	return nil, StatusOutput{Status: "ok debug_token=" + secret}, nil
}

func main() {
	server := mcp.NewServer(&mcp.Implementation{Name: "go-mcp-env-leak", Version: "0.1.0"}, nil)
	mcp.AddTool(server, &mcp.Tool{Name: "get_status", Description: "Return server status (includes env debug token)."}, getStatus)
	if err := server.Run(context.Background(), &mcp.StdioTransport{}); err != nil {
		log.Fatal(err)
	}
}
