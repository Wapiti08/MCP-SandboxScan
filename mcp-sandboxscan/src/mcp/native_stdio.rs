use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{bail, Result, Context};
use serde_json::Value;

use crate::mcp::driver::{McpCallPlan, McpDriver, McpDriverResult};
use crate::mcp::jsonrpc::{initialize_request, initialized_notification, tools_call_request};
use crate::mcp::transcript::{McpDirection, McpEvent, McpTranscript};
use crate::sandbox::exec_evidence::{ExecutionBackend, ExecutionEvidence};


pub struct NativeStdioMcpDriver {
    pub command: String,
    pub args: Vec<String>,
    pub current_dir: Option<PathBuf>,
    pub framing: StdioFraming,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdioFraming {
    Newline,
    ContentLength,
}

impl McpDriver for NativeStdioMcpDriver {
    // analayse through child process
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

        let mut transcript = McpTranscript {events: Vec::new() };
        // make request to real MCP server
        let initialize = initialize_request(1);
        send_message(&mut stdin, &initialize, self.framing)?;
        // record initial request
        record(&mut transcript, McpDirection::ClientToServer, &initialize);

        //get initial response
        let initialize_response = read_response_with_id(&mut reader, 1, self.framing)?;
        record(
            &mut transcript,
            McpDirection::ServerToClient,
            &initialize_response,
        );

        // get notification
        let initialized = initialized_notification();
        send_message(&mut stdin, &initialized, self.framing)?;
        record(&mut transcript, McpDirection::ClientToServer, &initialized);

        // make tool request
        let call = tools_call_request(2, &plan.tool_name, plan.arguments.clone());
        send_message(&mut stdin, &call, self.framing)?;
        record(&mut transcript, McpDirection::ClientToServer, &call);

        // process call response
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

trait CommandExt {
    // provide optional interface when no dir available
    fn current_dir_opt(&mut self, dir: Option<&PathBuf>) -> &mut Self;
}

// extension trail for built-in Command
impl CommandExt for Command {
    fn current_dir_opt(&mut self, dir: Option<&PathBuf>) ->  &mut Self {
        if let Some(dir) = dir {
            self.current_dir(dir)
        } else {
            self
        }
    }
}

// accept any type implements write trait
fn send_message(stdin: &mut impl Write, message: &Value, framing: StdioFraming) -> Result<()> {
    let line = serde_json::to_string(message)?;
    match framing {
        StdioFraming::Newline => {
            stdin.write_all(line.as_bytes())?;
            stdin.write_all(b"\n")?;
        }
        StdioFraming::ContentLength => {
            write!(stdin, "Content-Length: {}\r\n\r\n", line.len())?;
            stdin.write_all(line.as_bytes())?;
        }
    }
    stdin.flush()?;
    Ok(())
}

fn read_response_with_id(
    reader: &mut impl BufRead,
    expected_id: u64,
    framing: StdioFraming,
) -> Result<Value> {
    // line will change during the process
    let mut line = String::new();
    // indicate an infinite loop
    loop {
        let value = match framing {
            StdioFraming::Newline => read_newline_json(reader, &mut line)?,
            StdioFraming::ContentLength => read_content_length_json(reader, &mut line)?,
        };

        if value.get("id").and_then(|id| id.as_u64()) == Some(expected_id) {
            return Ok(value);
        }
    }
}

fn read_newline_json(reader: &mut impl BufRead, line: &mut String) -> Result<Value> {
    loop {
        line.clear();
        // return size, value wil go to line
        let n = reader.read_line(line)?;
        if n == 0 {
            bail!("MCP server closed stdout before JSON response");
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        return serde_json::from_str(trimmed)
            .with_context(|| format!("failed to parse MCP JSON line: {trimmed}"));
    }
}

fn read_content_length_json(reader: &mut impl BufRead, line: &mut String) -> Result<Value> {
    let mut content_length = None;

    loop {
        line.clear();
        let n = reader.read_line(line)?;
        if n == 0 {
            bail!("MCP server closed stdout before headers");
        }

        let header = line.trim_end_matches(['\r', '\n']);
        if header.is_empty() {
            break;
        }

        if let Some(value) = header.strip_prefix("Content-Length:") {
            content_length = Some(value.trim().parse::<usize>()?);
        }
    }

    let content_length = content_length.context("MCP response missing Content-Length header")?;
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body)?;

    serde_json::from_slice(&body).context("failed to parse MCP Content-Length JSON body")
}


fn record(transcript: &mut McpTranscript, direction: McpDirection, payload: &Value) {
    transcript.events.push(McpEvent {
        direction,
        method: payload
            .get("method")
            .and_then(|value| value.as_str())
            .map(|s| s.to_string()),
        payload: payload.clone(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::{ensure_rust_mcp_filesystem_repo, scan_subject};
    use crate::scan::prompt_sink::PromptSink;
    use crate::subject::SubjectManifest;
    use serde_json::json;

    #[test]
    fn native_stdio_driver_calls_mock_tool() {
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
    elif msg.get("id") == 2 and method == "tools/call":
        name = msg.get("params", {}).get("name")
        print(json.dumps({
            "jsonrpc": "2.0",
            "id": 2,
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
        assert_eq!(result.transcript.events.len(), 5);
        assert_eq!(
            result.transcript.events[0].method.as_deref(),
            Some("initialize")
        );
        assert_eq!(
            result.transcript.events[3].method.as_deref(),
            Some("tools/call")
        );
    }

    #[test]
    fn native_stdio_driver_calls_real_rust_mcp_filesystem() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        ensure_rust_mcp_filesystem_repo(&manifest_dir);

        let raw = std::fs::read_to_string("case_studies/rust-mcp-filesystem/subject.toml")
            .expect("read subject manifest");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");

        let data_dir = manifest_dir.join("data");
        std::fs::create_dir_all(&data_dir).expect("create allowed data dir");

        let result =
            scan_subject(&subject, &HashMap::new(), Some(&data_dir), 4096).expect("scan subject");

        assert_eq!(result.report.summary.num_sinks, 1);
        assert_eq!(result.report.summary.num_flows, 0);
        assert_eq!(result.report.mcp_transcript.as_ref().unwrap().events.len(), 5);
        assert!(matches!(
            result.report.sinks[0],
            PromptSink::McpToolResultText { .. }
        ));
        assert!(result.report.sinks[0]
            .as_text()
            .contains("Allowed directories"));
    }
}