use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::adapter::BuildArtifact;
use crate::collect::NetworkCollector;
use crate::mcp::driver::{McpCallPlan, McpDriver};
use crate::mcp::native_stdio::{NativeStdioMcpDriver, StdioFraming};
use crate::scan::mcp_scan::scan_mcp_driver_result;
use crate::scan::report::ScanReport;
use crate::subject::{Capability, SubjectManifest};
use crate::taint::source::TaintSource;

pub fn run_native_mcp_scan(
    subject: &SubjectManifest,
    artifact: &BuildArtifact,
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
    mcp_timeout: Option<Duration>,
) -> Result<ScanReport> {
    let BuildArtifact::NativeCommand { command, args } = artifact else {
        bail!("expected NativeCommand artifact for native MCP scan");
    };

    let mcp = subject.mcp.as_ref().context("missing [mcp] spec")?;

    let mut run_args = args.clone();
    let needs_data_dir_arg = subject
        .capabilities
        .iter()
        .any(|cap| matches!(cap, Capability::FileRead | Capability::FileWrite));
    if run_args.is_empty() && needs_data_dir_arg {
        let dir = data_dir.with_context(|| {
            format!(
                "subject {} requires --data-dir when [run].args is empty and file capabilities are declared",
                subject.name
            )
        })?;
        let abs_dir = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
        run_args.push(abs_dir.to_string_lossy().into_owned());
    }

    let network_collector = Arc::new(NetworkCollector::new());
    let proxy_port = network_collector
        .start_egress_proxy()
        .context("failed to start egress network proxy")?;

    let mut child_env = corpus_stub_env(env);
    let proxy_url = format!("http://127.0.0.1:{proxy_port}");
    child_env.insert("HTTP_PROXY".to_string(), proxy_url.clone());
    child_env.insert("HTTPS_PROXY".to_string(), proxy_url);
    child_env.insert("NO_PROXY".to_string(), "127.0.0.1,localhost".to_string());

    let plan = McpCallPlan {
        tool_name: mcp.tool.clone(),
        arguments: mcp.arguments.clone(),
    };

    let mut arg_variants = vec![run_args.clone()];
    if !run_args.iter().any(|a| a == "stdio") {
        let mut with_stdio = run_args.clone();
        with_stdio.push("stdio".to_string());
        arg_variants.push(with_stdio);
    }
    if !run_args.iter().any(|a| a == "--stdio") {
        let mut with_flag = run_args.clone();
        with_flag.push("--stdio".to_string());
        arg_variants.push(with_flag);
    }

    let mut last_err = None;
    let mut result = None;
    let commands: Vec<String> = if command == "bun" {
        vec!["bun".to_string(), "node".to_string()]
    } else {
        vec![command.clone()]
    };

    'outer: for cmd in commands {
        for args in arg_variants.clone() {
            let driver = NativeStdioMcpDriver {
                command: cmd.clone(),
                args,
                current_dir: Some(subject.source_dir.clone()),
                framing: StdioFraming::Newline,
                env: child_env.clone(),
                mcp_timeout,
            };
            match driver.call_tool(&plan) {
                Ok(r) => {
                    result = Some(r);
                    break 'outer;
                }
                Err(err) => last_err = Some(err),
            }
        }
    }

    let result = result.ok_or_else(|| {
        last_err.unwrap_or_else(|| anyhow::anyhow!("MCP scan failed with no attempts"))
    })?;

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

    sources.extend(network_collector.as_taint_sources());

    let mut report = scan_mcp_driver_result(result, sources);
    report.events.extend(network_collector.as_monitor_events());
    Ok(report)
}

fn corpus_stub_env(base: &HashMap<String, String>) -> HashMap<String, String> {
    let mut out = base.clone();
    const STUBS: &[(&str, &str)] = &[
        ("GITHUB_PERSONAL_ACCESS_TOKEN", "ghp_corpus_scan_stub"),
        ("GITHUB_TOKEN", "ghp_corpus_scan_stub"),
        ("OPENAI_API_KEY", "sk-corpus-scan-stub"),
        ("ANTHROPIC_API_KEY", "sk-ant-corpus-scan-stub"),
        ("FIGMA_API_KEY", "figd_corpus_scan_stub"),
        ("API_KEY", "corpus-scan-stub"),
    ];
    for (key, value) in STUBS {
        out.entry(key.to_string())
            .or_insert_with(|| value.to_string());
    }
    out
}
