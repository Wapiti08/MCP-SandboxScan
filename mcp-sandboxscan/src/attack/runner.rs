use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::attack::oracle::EndToEndOracle;
use crate::attack::scenario::{ExternalContent, ExternalContentScenario, derive_call_plan};
use crate::mcp::driver::McpDriver;
use crate::mcp::native_stdio::{NativeStdioMcpDriver, StdioFraming};
use crate::scan::mcp_scan::scan_mcp_driver_result;
use crate::scan::report::ScanReport;
use crate::taint::source::TaintSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEvidence {
    pub driver: String,
    pub tool_name: String,
    pub arguments: Value,
    pub derived_from_external_content: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityEvidence {
    pub authority: String,
    pub identifier: String,
    pub observation: String,
    pub inferred_used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinkEvidence {
    pub sink: String,
    pub canary_observed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalContentExperimentReport {
    pub experiment_id: String,
    pub description: String,
    pub scope_note: String,
    pub source: ExternalContent,
    pub trigger: TriggerEvidence,
    pub authority_use: AuthorityEvidence,
    pub sink: SinkEvidence,
    pub oracle: EndToEndOracle,
    pub scan_report: ScanReport,
}

/// Replays an external-content-triggered MCP invocation against a controlled
/// local server and evaluates the complete source -> trigger -> authority ->
/// sink chain.
pub fn run_external_content_experiment(
    scenario: &ExternalContentScenario,
    scenario_directory: &Path,
) -> Result<ExternalContentExperimentReport> {
    let plan = derive_call_plan(&scenario.source)
        .context("controlled agent could not derive a call from external content")?;
    let derived_from_external_content = true;

    let mut env = HashMap::new();
    env.insert(
        scenario.canary.env_key.clone(),
        scenario.canary.value.clone(),
    );

    let working_directory =
        resolve_working_directory(scenario_directory, &scenario.server.working_directory);
    let driver = NativeStdioMcpDriver {
        command: scenario.server.command.clone(),
        args: scenario.server.args.clone(),
        current_dir: Some(working_directory),
        framing: StdioFraming::Newline,
        env,
        mcp_timeout: Some(Duration::from_secs(10)),
    };
    let driver_result = driver
        .call_tool(&plan)
        .context("controlled MCP call failed")?;
    let sources = vec![TaintSource::EnvVar {
        key: scenario.canary.env_key.clone(),
        value: scenario.canary.value.clone(),
    }];
    let scan_report = scan_mcp_driver_result(driver_result, sources);

    let oracle = EndToEndOracle::evaluate(
        derived_from_external_content,
        &scenario.expected.tool_name,
        &scenario.canary.env_key,
        &scenario.canary.value,
        &scan_report,
    );
    let authority_use = AuthorityEvidence {
        authority: "environment-read".to_string(),
        identifier: scenario.canary.env_key.clone(),
        observation: "inferred from the unique environment canary in the MCP tool result"
            .to_string(),
        inferred_used: oracle.authority_use_inferred,
    };
    let sink = SinkEvidence {
        sink: "mcp-tool-result".to_string(),
        canary_observed: oracle.canary_reached_tool_output,
    };

    Ok(ExternalContentExperimentReport {
        experiment_id: scenario.id.clone(),
        description: scenario.description.clone(),
        scope_note: "Deterministic replay: validates the consequence of following external instructions, not the probability that an LLM will follow them."
            .to_string(),
        source: scenario.source.clone(),
        trigger: TriggerEvidence {
            driver: "deterministic-external-content-surrogate".to_string(),
            tool_name: plan.tool_name,
            arguments: plan.arguments,
            derived_from_external_content,
        },
        authority_use,
        sink,
        oracle,
        scan_report,
    })
}

fn resolve_working_directory(scenario_directory: &Path, configured: &str) -> PathBuf {
    let path = Path::new(configured);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        scenario_directory.join(path)
    }
}
