use serde::{Deserialize, Serialize};

use crate::scan::report::ScanReport;
use crate::taint::source::TaintSource;

use super::metrics::{Label, Verdict};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScenarioKind {
    Benign,
    EnvLeak,
    FileExfil,
    C2Beacon,
}

pub fn scenario_from_name(name: &str) -> ScenarioKind {
    let n = name.to_ascii_lowercase();
    if n.contains("env-leak") || n.contains("env_leak") {
        return ScenarioKind::EnvLeak;
    }
    if n.contains("file-exfil") || n.contains("file_exfil") {
        return ScenarioKind::FileExfil;
    }
    if n.contains("c2-beacon") || n.contains("c2_beacon") {
        return ScenarioKind::C2Beacon;
    }
    ScenarioKind::Benign
}

pub fn label_for_scenario(scenario: ScenarioKind) -> Label {
    match scenario {
        ScenarioKind::Benign => Label::Clean,
        ScenarioKind::EnvLeak | ScenarioKind::FileExfil | ScenarioKind::C2Beacon => {
            Label::Malicious
        }
    }
}

pub fn score_case(report: &ScanReport, scenario: ScenarioKind) -> (Verdict, String) {
    if let Some(code) = report.exec.exit_code {
        if code != 0 {
            return (Verdict::Error, format!("non-zero exit_code={code}"));
        }
    }

    match scenario {
        ScenarioKind::Benign => {
            if report.summary.has_external_to_prompt_flow {
                (
                    Verdict::Detected,
                    "unexpected external-to-sink flow in benign subject".to_string(),
                )
            } else {
                (Verdict::NotDetected, "no flows (expected)".to_string())
            }
        }
        ScenarioKind::EnvLeak => {
            let ok = report.summary.has_external_to_prompt_flow
                && report
                    .flows
                    .iter()
                    .any(|flow| flow.source_id.contains("EnvVar: DEMO_SECRET"));
            if ok {
                (
                    Verdict::Detected,
                    "flow from EnvVar: DEMO_SECRET".to_string(),
                )
            } else {
                (
                    Verdict::NotDetected,
                    "missing EnvVar: DEMO_SECRET flow".to_string(),
                )
            }
        }
        ScenarioKind::FileExfil => {
            let ok = report.summary.has_external_to_prompt_flow
                && report.flows.iter().any(|flow| {
                    flow.source_id.contains("secret.txt")
                        || flow.source_id.contains("FILE_TO_READ")
                        || flow.source_id.contains("FileRead:")
                });
            if ok {
                (Verdict::Detected, "flow from secret.txt".to_string())
            } else {
                (Verdict::NotDetected, "missing secret.txt flow".to_string())
            }
        }
        ScenarioKind::C2Beacon => {
            let has_network_src = report.sources.iter().any(|src| {
                matches!(
                    src,
                    TaintSource::NetworkConnect { .. } | TaintSource::HttpFetch { .. }
                )
            });
            let sink_mentions_beacon = report.sinks.iter().any(|sink| {
                let text = sink.as_text();
                text.contains("beacon") || text.contains("c2.evil") || text.contains("HTTP_FETCH:")
            });
            let ok = has_network_src
                && (report.summary.has_external_to_prompt_flow || sink_mentions_beacon);
            if ok {
                (
                    Verdict::Detected,
                    "network source with beacon evidence in flows or sinks".to_string(),
                )
            } else {
                (
                    Verdict::NotDetected,
                    "missing network/beacon evidence".to_string(),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::exec_evidence::{ExecutionBackend, ExecutionEvidence};
    use crate::scan::prompt_sink::PromptSink;
    use crate::scan::report::{ScanReport, Summary};
    use crate::taint::flow::FlowMatch;

    fn empty_report() -> ScanReport {
        ScanReport {
            exec: ExecutionEvidence {
                backend: ExecutionBackend::Wasm,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: Some(0),
                duration_ms: 0,
            },
            mcp_transcript: None,
            events: vec![],
            sources: vec![],
            sinks: vec![],
            flows: vec![],
            summary: Summary {
                num_sources: 0,
                num_sinks: 0,
                num_flows: 0,
                has_external_to_prompt_flow: false,
            },
        }
    }

    #[test]
    fn classifies_subject_names() {
        assert_eq!(scenario_from_name("rust-env-leak"), ScenarioKind::EnvLeak);
        assert_eq!(scenario_from_name("ts-mcp-env-leak"), ScenarioKind::EnvLeak);
        assert_eq!(scenario_from_name("go-benign"), ScenarioKind::Benign);
        assert_eq!(scenario_from_name("ts-mcp-echo"), ScenarioKind::Benign);
    }

    #[test]
    fn scores_env_leak_flow() {
        let mut report = empty_report();
        report.summary.has_external_to_prompt_flow = true;
        report.summary.num_flows = 1;
        report.flows.push(FlowMatch {
            source_id: "EnvVar: DEMO_SECRET".to_string(),
            sink_type: "tool-return".to_string(),
            snippet: "SEKRET".to_string(),
            confidence: "high".to_string(),
        });

        let (verdict, _) = score_case(&report, ScenarioKind::EnvLeak);
        assert_eq!(verdict, Verdict::Detected);
    }

    #[test]
    fn scores_benign_false_positive() {
        let mut report = empty_report();
        report.summary.has_external_to_prompt_flow = true;
        let (verdict, _) = score_case(&report, ScenarioKind::Benign);
        assert_eq!(verdict, Verdict::Detected);
    }

    #[test]
    fn scores_c2_from_network_source() {
        let mut report = empty_report();
        report.sources.push(TaintSource::HttpFetch {
            url: "https://c2.evil.example/beacon".to_string(),
            content: String::new(),
        });
        report.sinks.push(PromptSink::ToolReturnLeaf {
            path: "$".to_string(),
            value: "beacon ok".to_string(),
        });

        let (verdict, _) = score_case(&report, ScenarioKind::C2Beacon);
        assert_eq!(verdict, Verdict::Detected);
    }
}
