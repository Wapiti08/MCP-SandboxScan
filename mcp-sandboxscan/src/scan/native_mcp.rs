use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::adapter::BuildArtifact;
use crate::collect::NetworkCollector;
use crate::mcp::driver::{McpCallPlan, McpDriver};
use crate::mcp::explore::ExplorationConfig;
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
    exploration: &ExplorationConfig,
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
    let exploration = prepare_exploration(subject, exploration, &mut child_env)?;
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
            let scan_result = if exploration.enabled {
                driver.call_tool_with_exploration(&plan, &exploration)
            } else {
                driver.call_tool(&plan)
            };
            match scan_result {
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

    if exploration.enabled {
        sources.push(TaintSource::EnvVar {
            key: "MCP_SANDBOXSCAN_CANARY".to_string(),
            value: exploration.env_canary.clone(),
        });

        if let Some(path) = &exploration.file_canary_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                sources.push(TaintSource::FileRead {
                    path: path.clone(),
                    content,
                });
            }
        }
    }

    if exploration.enabled {
        sources.extend(exploration_tool_input_sources(&result.tool_result_payload));
    }

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

fn exploration_tool_input_sources(payload: &Value) -> Vec<TaintSource> {
    let Some(results) = payload
        .get("exploration_results")
        .and_then(|results| results.as_array())
    else {
        return Vec::new();
    };

    let mut sources = Vec::new();
    for item in results {
        let tool = item
            .get("tool")
            .and_then(|tool| tool.as_str())
            .unwrap_or("unknown")
            .to_string();
        if let Some(arguments) = item.get("arguments") {
            collect_tool_input_leaves(&tool, "$.arguments", arguments, &mut sources);
        }
    }
    sources
}

fn collect_tool_input_leaves(
    tool: &str,
    path: &str,
    value: &Value,
    sources: &mut Vec<TaintSource>,
) {
    match value {
        Value::String(content) => {
            if content.contains("MCP_INPUT_CANARY") || content.contains("MCP_FILE_CANARY") {
                sources.push(TaintSource::ToolInput {
                    tool: tool.to_string(),
                    path: path.to_string(),
                    content: content.clone(),
                });
            }
        }
        Value::Array(items) => {
            for (idx, item) in items.iter().enumerate() {
                collect_tool_input_leaves(tool, &format!("{path}[{idx}]"), item, sources);
            }
        }
        Value::Object(map) => {
            for (key, item) in map {
                collect_tool_input_leaves(tool, &format!("{path}.{key}"), item, sources);
            }
        }
        _ => {}
    }
}

fn prepare_exploration(
    subject: &SubjectManifest,
    exploration: &ExplorationConfig,
    child_env: &mut HashMap<String, String>,
) -> Result<ExplorationConfig> {
    if !exploration.enabled {
        return Ok(exploration.clone());
    }

    let slug = subject
        .name
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    let env_canary = if exploration.env_canary.is_empty() {
        format!("MCP_ENV_CANARY_{slug}")
    } else {
        exploration.env_canary.clone()
    };
    child_env.insert("MCP_SANDBOXSCAN_CANARY".to_string(), env_canary.clone());
    child_env.insert("MCP_ENV_CANARY".to_string(), env_canary.clone());

    let input_canary = if exploration.input_canary.is_empty() {
        format!("MCP_INPUT_CANARY_{slug}")
    } else {
        exploration.input_canary.clone()
    };

    let canary_dir = std::env::temp_dir().join("mcp-sandboxscan-canaries");
    std::fs::create_dir_all(&canary_dir)?;
    let canary_path = canary_dir.join(format!("{slug}.txt"));
    let file_canary = format!("MCP_FILE_CANARY_{slug}");
    std::fs::write(&canary_path, &file_canary)?;

    Ok(ExplorationConfig {
        enabled: true,
        max_tools: exploration.max_tools,
        env_canary,
        input_canary,
        file_canary_path: Some(canary_path.to_string_lossy().into_owned()),
    })
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
