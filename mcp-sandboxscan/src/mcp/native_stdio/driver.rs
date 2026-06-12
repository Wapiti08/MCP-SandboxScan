use std::collections::HashMap;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

use crate::mcp::driver::{McpCallPlan, McpDriver, McpDriverResult};
use crate::mcp::jsonrpc::{initialize_request, initialized_notification, tools_call_request};
use crate::mcp::transcript::{McpDirection, McpTranscript};
use crate::sandbox::exec_evidence::{ExecutionBackend, ExecutionEvidence};

use super::protocol::{
    read_response_with_id, record, send_message, CommandExt, StdioFraming,
};

pub struct NativeStdioMcpDriver {
    pub command: String,
    pub args: Vec<String>,
    pub current_dir: Option<PathBuf>,
    pub framing: StdioFraming,
    pub env: HashMap<String, String>,
}

impl McpDriver for NativeStdioMcpDriver {
    fn call_tool(&self, plan: &McpCallPlan) -> Result<McpDriverResult> {
        let started = std::time::Instant::now();
        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args)
            .current_dir_opt(self.current_dir.as_ref())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        for (key, value) in &self.env {
            cmd.env(key, value);
        }

        let mut child = cmd
            .spawn()
            .with_context(|| format!("failed to spawn MCP server `{}`", self.command))?;

        let mut stdin = child.stdin.take().context("failed to open MCP stdin")?;
        let stdout = child.stdout.take().context("failed to open MCP stdout")?;
        let mut reader = BufReader::new(stdout);

        let mut transcript = McpTranscript { events: Vec::new() };

        let initialize = initialize_request(1);
        send_message(&mut stdin, &initialize, self.framing)?;
        record(&mut transcript, McpDirection::ClientToServer, &initialize);

        let initialize_response = read_response_with_id(&mut reader, 1, self.framing)?;
        record(
            &mut transcript,
            McpDirection::ServerToClient,
            &initialize_response,
        );

        let initialized = initialized_notification();
        send_message(&mut stdin, &initialized, self.framing)?;
        record(&mut transcript, McpDirection::ClientToServer, &initialized);

        let call = tools_call_request(2, &plan.tool_name, plan.arguments.clone());
        send_message(&mut stdin, &call, self.framing)?;
        record(&mut transcript, McpDirection::ClientToServer, &call);

        let call_response = read_response_with_id(&mut reader, 2, self.framing)?;
        record(&mut transcript, McpDirection::ServerToClient, &call_response);

        let tool_result_payload = call_response
            .get("result")
            .cloned()
            .context("tools/call response missing result")?;
        let _ = child.kill();
        let _ = child.wait();
        Ok(McpDriverResult {
            exec: ExecutionEvidence {
                backend: ExecutionBackend::NativeStdio,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: None,
                duration_ms: started.elapsed().as_millis(),
            },
            transcript,
            tool_result_payload,
        })
    }
}
