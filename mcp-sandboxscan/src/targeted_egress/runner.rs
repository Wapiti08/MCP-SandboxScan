use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::mcp::explore::ExplorationConfig;
use crate::monitor::event::MonitorEventKind;
use crate::pipeline::{ScanLimits, scan_subject_with_limits};
use crate::subject::{McpSpec, RunSpec, SubjectManifest};

use super::model::{CaseStatus, ExpectedOutcome, TargetedCase, TargetedCaseReport, TargetedPlan};

pub fn load_plan(path: &Path) -> Result<TargetedPlan> {
    let raw = fs::read_to_string(path).with_context(|| format!("read plan {}", path.display()))?;

    serde_json::from_str(&raw).with_context(|| format!("parse plan {}", path.display()))
}

fn slug(repo_id: &str) -> String {
    repo_id.replace('/', "__")
}

fn load_subject(
    case: &TargetedCase,
    workspace: &Path,
    subjects_root: &Path,
) -> Result<SubjectManifest> {
    let manifest_path = workspace.join(&case.manifest);
    let raw = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;

    let mut subject: SubjectManifest =
        toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))?;

    // replace original source_dir with a path under subjects_root, using a slugified version of the repo_id
    subject.source_dir = subjects_root.join(slug(&case.repo_id));

    subject.mcp = Some(McpSpec {
        tool: case.tool.clone(),
        arguments: case.arguments.clone(),
    });

    Ok(subject)
}

pub fn prepare_case(case: &TargetedCase, workspace: &Path, subjects_root: &Path) -> Result<()> {
    if !case.enabled {
        return Ok(());
    }

    let case_slug = slug(&case.repo_id);
    let source = workspace
        .join("mcp-sandboxscan/corpus/clones")
        .join(&case_slug);
    let target = subjects_root.join(&case_slug);

    if !source.exists() {
        bail!("source clone missing: {}", source.display());
    }

    fs::create_dir_all(&target)?;

    // exclude macOS node_modules、venv、target directories to avoid copying unnecessary files
    let source_with_slash = format!("{}/", source.display());
    let status = Command::new("rsync")
        .args([
            "-a",
            "--delete",
            "--exclude=node_modules",
            "--exclude=.venv",
            "--exclude=target",
            &source_with_slash,
            &target.to_string_lossy(),
        ])
        .status()
        .context("run rsync")?;

    if !status.success() {
        bail!("rsync failed for {}", case.repo_id);
    }

    let subject = load_subject(case, workspace, subjects_root)?;

    if let Some(build) = subject.build {
        let status = Command::new(&build.command)
            .args(&build.args)
            .current_dir(&subject.source_dir)
            .env("PIP_DISABLE_PIP_VERSION_CHECK", "1")
            .env("PIP_NO_PROGRESS_BAR", "1")
            .status()
            .with_context(|| format!("prepare {}", case.repo_id))?;

        if !status.success() {
            bail!("build failed for {}", case.repo_id);
        }
    }

    for prepare in &case.prepare_commands {
        let status = Command::new(&prepare.command)
            .args(&prepare.args)
            .current_dir(&subject.source_dir)
            .envs(&case.environment)
            .env("PIP_DISABLE_PIP_VERSION_CHECK", "1")
            .env("PIP_NO_PROGRESS_BAR", "1")
            .status()
            .with_context(|| {
                format!(
                    "run prepare command `{}` for {}",
                    prepare.command, case.repo_id
                )
            })?;

        if !status.success() {
            bail!(
                "prepare command `{}` failed for {}",
                prepare.command,
                case.repo_id
            );
        }
    }

    Ok(())
}

pub fn run_case(
    case: &TargetedCase,
    workspace: &Path,
    subjects_root: &Path,
    results_dir: &Path,
) -> Result<TargetedCaseReport> {
    if !case.enabled {
        return Ok(TargetedCaseReport {
            repo_id: case.repo_id.clone(),
            tool: case.tool.clone(),
            status: CaseStatus::Excluded,
            expected: format!("{:?}", case.expected),
            proxy_network_events: 0,
            socket_connect_attempts: 0,
            socket_evidence: Vec::new(),
            expected_ports: case.expected_ports.clone(),
            error: None,
            exclusion_reason: case.exclusion_reason.clone(),
            scan_report: None,
        });
    }

    fs::create_dir_all(results_dir)?;
    let results_dir = fs::canonicalize(results_dir)?;

    let mut subject = load_subject(case, workspace, subjects_root)?;

    // prepare stage has successfully built the tool, no npm/pip install during running without internet
    subject.build = None;

    let original_run = subject
        .run
        .clone()
        .context("subject has no run specification")?;

    let trace_path = results_dir.join(format!("{}.connect.log", slug(&case.repo_id)));
    if trace_path.exists() {
        fs::remove_file(&trace_path)
            .with_context(|| format!("remove stale trace {}", trace_path.display()))?;
    }

    let mut wrapped_args = vec![
        "-f".to_string(),
        "-qq".to_string(),
        "-A".to_string(),
        "-e".to_string(),
        "trace=connect".to_string(),
        "-s".to_string(),
        "256".to_string(),
        "-o".to_string(),
        trace_path.to_string_lossy().into_owned(),
        "--".to_string(),
        original_run.command,
    ];

    wrapped_args.extend(original_run.args);
    wrapped_args.extend(case.run_args_append.clone());

    subject.run = Some(RunSpec {
        command: "strace".to_string(),
        args: wrapped_args,
    });

    let limits = ScanLimits {
        build_timeout: Some(Duration::from_secs(30)),
        mcp_timeout: Some(Duration::from_secs(30)),
        exploration: ExplorationConfig::disabled(),
    };

    let scan = scan_subject_with_limits(&subject, &case.environment, None, 512_000, limits);

    let socket_evidence = read_socket_evidence(&trace_path, &case.expected_ports);
    let socket_connect_attempts = socket_evidence.len();

    match scan {
        Ok(result) => {
            let proxy_network_events = result
                .report
                .events
                .iter()
                .filter(|event| {
                    matches!(
                        event.kind,
                        MonitorEventKind::NetworkRequest
                            | MonitorEventKind::NetworkConnectAttempt
                            | MonitorEventKind::NetworkConnectAllowed
                            | MonitorEventKind::NetworkConnectDenied
                    )
                })
                .count();

            let status = if socket_connect_attempts > 0 && proxy_network_events == 0 {
                CaseStatus::MonitoringBlindSpot
            } else if socket_connect_attempts > 0 || proxy_network_events > 0 {
                CaseStatus::NetworkAttemptObserved
            } else if matches!(case.expected, ExpectedOutcome::PreconditionFailure) {
                CaseStatus::PreconditionFailed
            } else {
                CaseStatus::NoNetworkObserved
            };

            Ok(TargetedCaseReport {
                repo_id: case.repo_id.clone(),
                tool: case.tool.clone(),
                status,
                expected: format!("{:?}", case.expected),
                proxy_network_events,
                socket_connect_attempts,
                socket_evidence,
                expected_ports: case.expected_ports.clone(),
                error: None,
                exclusion_reason: None,
                scan_report: Some(result.report),
            })
        }

        Err(error) => {
            let status = if socket_connect_attempts > 0 {
                CaseStatus::MonitoringBlindSpot
            } else {
                CaseStatus::PreconditionFailed
            };

            Ok(TargetedCaseReport {
                repo_id: case.repo_id.clone(),
                tool: case.tool.clone(),
                status,
                expected: format!("{:?}", case.expected),
                proxy_network_events: 0,
                socket_connect_attempts,
                socket_evidence,
                expected_ports: case.expected_ports.clone(),
                error: Some(format!("{error:#}")),
                exclusion_reason: None,
                scan_report: None,
            })
        }
    }
}

fn read_socket_evidence(path: &Path, expected_ports: &[u16]) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|line| {
            let is_ip_connect = line.contains("connect(")
                && (line.contains("AF_INET") || line.contains("AF_INET6"));
            let matches_expected_port = expected_ports.is_empty()
                || expected_ports.iter().any(|port| {
                    line.contains(&format!("sin_port=htons({port})"))
                        || line.contains(&format!("sin6_port=htons({port})"))
                });
            is_ip_connect && matches_expected_port
        })
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_replaces_repository_separator() {
        assert_eq!(slug("owner/repo"), "owner__repo");
    }

    #[test]
    fn socket_evidence_filters_unrelated_ports_and_unix_sockets() {
        let trace_path = std::env::temp_dir().join(format!(
            "sandscope-targeted-egress-trace-{}.log",
            std::process::id()
        ));
        let trace = r#"100 connect(3, {sa_family=AF_UNIX, sun_path="/tmp/browser.sock"}, 20) = 0
101 connect(4, {sa_family=AF_INET, sin_port=htons(9222), sin_addr=inet_addr("127.0.0.1")}, 16) = 0
102 connect(5, {sa_family=AF_INET, sin_port=htons(18080), sin_addr=inet_addr("127.0.0.1")}, 16) = -1 ECONNREFUSED
"#;
        fs::write(&trace_path, trace).expect("write trace fixture");

        let evidence = read_socket_evidence(&trace_path, &[18080]);
        fs::remove_file(&trace_path).ok();

        assert_eq!(evidence.len(), 1);
        assert!(evidence[0].contains("htons(18080)"));
    }
}
