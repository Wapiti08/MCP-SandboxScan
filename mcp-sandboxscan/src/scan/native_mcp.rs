use std::collections::HashMap;
use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::adapter::BuildArtifact;
use crate::mcp::driver::{McpCallPlan, McpDriver};
use crate::mcp::native_stdio::{NativeStdioMcpDriver, StdioFraming};
use crate::scan::mcp_scan::scan_mcp_driver_result;
use crate::scan::report::ScanReport;
use crate::subject::SubjectManifest;
use crate::taint::source::TaintSource;

pub fn run_native_mcp_scan(
    subject: &SubjectManifest,
    artifact: &BuildArtifact,
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
) -> Result<ScanReport> {
    let BuildArtifact::NativeCommand { command, args } = artifact else {
        bail!("expected NativeCommand artifact for native MCP scan");
    };

    let mcp = subject.mcp.as_ref().context("missing [mcp] spec")?;

    let mut run_args = args.clone();
    if run_args.is_empty() {
        let dir = data_dir.with_context(|| {
            format!(
                "subject {} requires --data-dir when [run].args is empty",
                subject.name
            )
        })?;
        let abs_dir = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
        run_args.push(abs_dir.to_string_lossy().into_owned());
    }

    let driver = NativeStdioMcpDriver {
        command: command.clone(),
        args: run_args,
        current_dir: Some(subject.source_dir.clone()),
        framing: StdioFraming::Newline,
        env: env.clone(),
    };

    let plan = McpCallPlan {
        tool_name: mcp.tool.clone(),
        arguments: mcp.arguments.clone(),
    };

    let result = driver
        .call_tool(&plan)
        .with_context(|| format!("failed MCP tools/call for {}", mcp.tool))?;

    let mut sources: Vec<TaintSource> = env
        .iter()
        .map(|(key, value)| TaintSource::EnvVar {
            key: key.clone(),
            value: value.clone(),
        })
        .collect();

    if let Some(dir) = data_dir {
        let secret_path = dir.join("secret.txt");
        if secret_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&secret_path) {
                sources.push(TaintSource::FileRead {
                    path: secret_path.to_string_lossy().into_owned(),
                    content,
                });
            }
        }
    }

    Ok(scan_mcp_driver_result(result, sources))
}
