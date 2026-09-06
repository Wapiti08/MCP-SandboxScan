use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::scan::report::ScanReport;
use crate::subject::RunSpec;

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct TargetedPlan {
    pub cases: Vec<TargetedCase>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TargetedCase {
    pub repo_id: String,
    pub manifest: PathBuf,

    #[serde(default = "default_enabled")]
    pub enabled: bool,

    pub tool: String,

    #[serde(default)]
    pub arguments: Value,

    #[serde(default)]
    pub environment: HashMap<String, String>,

    #[serde(default)]
    pub run_args_append: Vec<String>,

    /// Case-specific setup needed after the manifest's normal build command.
    /// These commands run only in the network-enabled prepare phase.
    #[serde(default)]
    pub prepare_commands: Vec<RunSpec>,

    /// Only socket connections to these ports count as evidence for this case.
    /// An empty list accepts all AF_INET/AF_INET6 connect attempts.
    #[serde(default)]
    pub expected_ports: Vec<u16>,

    pub expected: ExpectedOutcome,

    #[serde(default)]
    pub exclusion_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedOutcome {
    NetworkAttempt,
    MonitoringBlindSpot,
    ClassifierReview,
    PreconditionFailure,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseStatus {
    NetworkAttemptObserved,
    MonitoringBlindSpot,
    NoNetworkObserved,
    PreconditionFailed,
    Excluded,
}

#[derive(Debug, Serialize)]
pub struct TargetedCaseReport {
    pub repo_id: String,
    pub tool: String,
    pub status: CaseStatus,
    pub expected: String,

    pub proxy_network_events: usize,
    pub socket_connect_attempts: usize,
    pub socket_evidence: Vec<String>,
    pub expected_ports: Vec<u16>,

    pub error: Option<String>,
    pub exclusion_reason: Option<String>,
    pub scan_report: Option<ScanReport>,
}

#[derive(Debug, Serialize)]
pub struct TargetedRunReport {
    pub cases: Vec<TargetedCaseReport>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_defaults_enabled_and_accepts_expected_ports() {
        let plan: TargetedPlan = serde_json::from_value(serde_json::json!({
            "cases": [{
                "repo_id": "owner/repo",
                "manifest": "corpus/manifests/owner__repo.toml",
                "tool": "read_only_tool",
                "arguments": {},
                "expected_ports": [443],
                "expected": "network_attempt"
            }]
        }))
        .expect("parse targeted plan");

        assert_eq!(plan.cases.len(), 1);
        assert!(plan.cases[0].enabled);
        assert_eq!(plan.cases[0].expected_ports, vec![443]);
        assert!(plan.cases[0].prepare_commands.is_empty());
    }
}
