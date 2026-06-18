use std::collections::HashMap;

use serde_json::json;

use crate::mcp::driver::{McpCallPlan, McpDriver};
use crate::mcp::native_stdio::{NativeStdioMcpDriver, StdioFraming};

#[test]
fn driver_calls_inline_python_mock_tool() {
    let script = r#"
import json
import sys

for line in sys.stdin:
    msg = json.loads(line)
    method = msg.get("method")
    if msg.get("id") == 1 and method == "initialize":
        print(json.dumps({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "protocolVersion": "2025-06-18",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "mock-mcp", "version": "0.1.0"}
            }
        }), flush=True)
    elif msg.get("id") == 2 and method == "tools/list":
        print(json.dumps({
            "jsonrpc": "2.0",
            "id": 2,
            "result": {
                "tools": [
                    {"name": "echo", "description": "mock echo", "inputSchema": {"type": "object"}}
                ]
            }
        }), flush=True)
    elif msg.get("id") == 3 and method == "tools/call":
        name = msg.get("params", {}).get("name")
        print(json.dumps({
            "jsonrpc": "2.0",
            "id": 3,
            "result": {
                "content": [
                    {"type": "text", "text": f"mock result from {name}"}
                ],
                "isError": False
            }
        }), flush=True)
"#;

    let driver = NativeStdioMcpDriver {
        command: "python3".to_string(),
        args: vec!["-u".to_string(), "-c".to_string(), script.to_string()],
        current_dir: None,
        framing: StdioFraming::Newline,
        env: HashMap::new(),
        mcp_timeout: None,
    };
    let plan = McpCallPlan {
        tool_name: "echo".to_string(),
        arguments: json!({"message": "hello"}),
    };

    let result = driver.call_tool(&plan).expect("call mock MCP tool");

    assert_eq!(
        result.tool_result_payload["content"][0]["text"],
        "mock result from echo"
    );
    assert_eq!(result.transcript.events.len(), 7);
    assert_eq!(
        result.transcript.events[0].method.as_deref(),
        Some("initialize")
    );
    assert_eq!(
        result.transcript.events[5].method.as_deref(),
        Some("tools/call")
    );
}
