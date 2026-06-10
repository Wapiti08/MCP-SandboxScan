use serde_json::{json, Value};

/*
generate MCP messages --- message builder from MCP client side
*/

pub fn initialize_request(id: u64) -> Value {
    // variables and expressions can be interpolated directly into the JSON value 
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {
                "name": "mcp-sandboxscan",
                "version": "0.1.0"
            }
        }
    })
}

pub fn initialized_notification() -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    })
}

// ask server about available tool list
pub fn tools_list_request(id: u64) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/list",
        "params": {}
    })
}
 
pub fn tools_call_request(id: u64, name: &str, arguments: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": {
            "name": name,
            "arguments": arguments
        }
    })
}
