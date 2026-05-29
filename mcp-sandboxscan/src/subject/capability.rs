use serde::{Deserialize, Serialize};

// define the capabilities that MCP/tool may need
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    Env,
    FileRead,
    FileWrite,
    Network,
    Subprocess,
    Stdio,
    McpProtocol,
}

