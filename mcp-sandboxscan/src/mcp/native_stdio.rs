use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{bail, Result, Context};
use serde_json::Value;

use crate::mcp::driver::{McpCallPlan, McpDriver, McpDriverResult};
use crate::mcp::jsonrpc::{initialize_request, initialized_notification, tools_call_request};
use crate::mcp::transcript::{McpDirection, McpEvent, McpTranscript};

pub struct NativeStdioMcpDriver {
    pub command: String,
    pub args: Vec<String>,
    pub current_dir: Option<PathBuf>,
}

impl McpDriver for NativeStdioMcpDriver {
    // analayse through child process
    fn call_tool(&self, plan: &McpCallPlan) -> Result<McpDriverResult> {
        let mut child = Command::new(&self.command)
            .args(&self.args)
            .current_dir_opt(self.current_dir.as_ref())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("failed to spawn MCP server `{}`", self.command))?;

        let mut stdin = child.stdin.take().context("failed to open MCP stdin")?;
        let stdout = child.stdout.take().context("failed to open MCP stdout")?;
        let mut reader = BufReader::new(stdout);

        
    
    }
}

