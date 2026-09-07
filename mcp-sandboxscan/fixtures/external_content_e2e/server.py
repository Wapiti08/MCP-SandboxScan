#!/usr/bin/env python3
"""Dependency-free MCP fixture for the external-content end-to-end replay."""

import json
import os
import sys


TOOL_NAME = "get_environment_status"


def response(request_id, result):
    return {"jsonrpc": "2.0", "id": request_id, "result": result}


def handle(message):
    method = message.get("method")
    request_id = message.get("id")

    if method == "initialize":
        return response(
            request_id,
            {
                "protocolVersion": "2025-06-18",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "external-content-e2e", "version": "0.1.0"},
            },
        )
    if method == "tools/list":
        return response(
            request_id,
            {
                "tools": [
                    {
                        "name": TOOL_NAME,
                        "description": "Return detailed process environment diagnostics.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "detail": {"type": "string", "enum": ["summary", "full"]}
                            },
                            "required": ["detail"],
                        },
                    }
                ]
            },
        )
    if method == "tools/call":
        params = message.get("params", {})
        if params.get("name") != TOOL_NAME:
            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "error": {"code": -32602, "message": "unknown tool"},
            }

        # This is the authority use under test. The canary is available to the
        # MCP process but is absent from the external issue and call arguments.
        _secret = os.environ.get("MCP_E2E_SECRET", "<missing MCP_E2E_SECRET>")
        detail = params.get("arguments", {}).get("detail", "summary")
        return response(
            request_id,
            {
                "content": [
                    {
                        "type": "text",
                        "text": f"status=ok detail={detail} diagnostic_token=<redacted>",
                    }
                ],
                "isError": False,
            },
        )
    return None


for line in sys.stdin:
    try:
        message = json.loads(line)
        result = handle(message)
        if result is not None:
            print(json.dumps(result, separators=(",", ":")), flush=True)
    except Exception as error:  # keep protocol failures observable to the driver
        print(
            json.dumps(
                {
                    "jsonrpc": "2.0",
                    "id": None,
                    "error": {"code": -32603, "message": str(error)},
                }
            ),
            flush=True,
        )
