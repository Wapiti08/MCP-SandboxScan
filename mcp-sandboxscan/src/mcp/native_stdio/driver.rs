use std::collections::HashMap;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::mcp::driver::{McpCallPlan, McpDriver, McpDriverResult};
use crate::mcp::explore::{ExplorationConfig, build_exploration_plans};
use crate::mcp::jsonrpc::{
    initialize_request, initialized_notification, tools_call_request, tools_list_request,
};
use crate::mcp::transcript::{McpDirection, McpTranscript};
use crate::sandbox::exec_evidence::{ExecutionBackend, ExecutionEvidence};

use super::protocol::{CommandExt, StdioFraming, read_response_with_id, record, send_message};

fn pick_tool_name(tools_response: &serde_json::Value, preferred: &str) -> String {
    let Some(tools) = tools_response
        .get("result")
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array())
    else {
        return preferred.to_string();
    };

    if tools
        .iter()
        .any(|t| t.get("name").and_then(|n| n.as_str()) == Some(preferred))
    {
        return preferred.to_string();
    }

    tools
        .first()
        .and_then(|t| t.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or(preferred)
        .to_string()
}

#[derive(Clone)]
pub struct NativeStdioMcpDriver {
    pub command: String,
    pub args: Vec<String>,
    pub current_dir: Option<PathBuf>,
    pub framing: StdioFraming,
    pub env: HashMap<String, String>,
    pub mcp_timeout: Option<Duration>,
}

impl McpDriver for NativeStdioMcpDriver {
    fn call_tool(&self, plan: &McpCallPlan) -> Result<McpDriverResult> {
        let Some(timeout) = self.mcp_timeout else {
            return self.call_tool_inner(plan, None);
        };

        let driver = self.clone();
        let plan = plan.clone();
        let child_holder: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
        let child_holder_worker = child_holder.clone();
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let result = driver.call_tool_inner(&plan, Some(child_holder_worker));
            let _ = tx.send(result);
        });

        match rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Ok(mut slot) = child_holder.lock() {
                    if let Some(mut child) = slot.take() {
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                }
                bail!("MCP server timed out after {}s", timeout.as_secs());
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                bail!("MCP scan thread exited unexpectedly");
            }
        }
    }
}

impl NativeStdioMcpDriver {
    pub fn call_tool_with_exploration(
        &self,
        plan: &McpCallPlan,
        exploration: &ExplorationConfig,
    ) -> Result<McpDriverResult> {
        let Some(timeout) = self.mcp_timeout else {
            return self.call_tools_inner(plan, Some(exploration), None);
        };

        let driver = self.clone();
        let plan = plan.clone();
        let exploration = exploration.clone();
        let child_holder: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
        let child_holder_worker = child_holder.clone();
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let result =
                driver.call_tools_inner(&plan, Some(&exploration), Some(child_holder_worker));
            let _ = tx.send(result);
        });

        match rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Ok(mut slot) = child_holder.lock() {
                    if let Some(mut child) = slot.take() {
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                }
                bail!("MCP server timed out after {}s", timeout.as_secs());
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                bail!("MCP scan thread exited unexpectedly");
            }
        }
    }

    fn call_tool_inner(
        &self,
        plan: &McpCallPlan,
        child_holder: Option<Arc<Mutex<Option<Child>>>>,
    ) -> Result<McpDriverResult> {
        self.call_tools_inner(plan, None, child_holder)
    }

    fn call_tools_inner(
        &self,
        plan: &McpCallPlan,
        exploration: Option<&ExplorationConfig>,
        child_holder: Option<Arc<Mutex<Option<Child>>>>,
    ) -> Result<McpDriverResult> {
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

        let mut owned_child = if let Some(holder) = child_holder.as_ref() {
            if let Ok(mut slot) = holder.lock() {
                *slot = Some(child);
            }
            None
        } else {
            Some(child)
        };

        let mut reader = BufReader::new(stdout);

        let mut transcript = McpTranscript { events: Vec::new() };

        let result = (|| -> Result<McpDriverResult> {
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

            let tools_list = tools_list_request(2);
            send_message(&mut stdin, &tools_list, self.framing)?;
            record(&mut transcript, McpDirection::ClientToServer, &tools_list);

            let tools_response = read_response_with_id(&mut reader, 2, self.framing)?;
            record(
                &mut transcript,
                McpDirection::ServerToClient,
                &tools_response,
            );

            let exploration_enabled = matches!(exploration, Some(config) if config.enabled);
            let plans = match exploration {
                Some(config) if config.enabled => {
                    let mut plans = build_exploration_plans(&tools_response, config);
                    if plans.is_empty() {
                        plans.push(McpCallPlan {
                            tool_name: pick_tool_name(&tools_response, &plan.tool_name),
                            arguments: plan.arguments.clone(),
                        });
                    }
                    plans
                }
                _ => vec![McpCallPlan {
                    tool_name: pick_tool_name(&tools_response, &plan.tool_name),
                    arguments: plan.arguments.clone(),
                }],
            };

            let mut results = Vec::new();
            for (idx, selected_plan) in plans.iter().enumerate() {
                let request_id = 3 + idx as u64;
                let call = tools_call_request(
                    request_id,
                    &selected_plan.tool_name,
                    selected_plan.arguments.clone(),
                );
                send_message(&mut stdin, &call, self.framing)?;
                record(&mut transcript, McpDirection::ClientToServer, &call);

                let call_response = read_response_with_id(&mut reader, request_id, self.framing)?;
                record(
                    &mut transcript,
                    McpDirection::ServerToClient,
                    &call_response,
                );

                let result = if let Some(result) = call_response.get("result").cloned() {
                    result
                } else if exploration_enabled {
                    serde_json::json!({
                        "isError": true,
                        "error": call_response.get("error").cloned().unwrap_or_else(|| {
                            serde_json::json!({
                                "message": "tools/call response missing result"
                            })
                        })
                    })
                } else {
                    bail!("tools/call response missing result");
                };
                results.push(serde_json::json!({
                    "tool": selected_plan.tool_name,
                    "arguments": selected_plan.arguments.clone(),
                    "result": result,
                }));
            }

            let tool_result_payload = if results.len() == 1 && exploration.is_none() {
                results
                    .into_iter()
                    .next()
                    .and_then(|r| r.get("result").cloned())
                    .context("tools/call response missing result")?
            } else {
                serde_json::json!({ "exploration_results": results })
            };

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
        })();

        if let Some(holder) = child_holder {
            if let Ok(mut slot) = holder.lock() {
                if let Some(mut child) = slot.take() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        } else if let Some(mut child) = owned_child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        result
    }
}
