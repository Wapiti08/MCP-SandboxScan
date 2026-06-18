use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::pipeline::case_study::{default_env_for_subject, resolve_data_dir};
use crate::pipeline::scan_subject;
use crate::scan::compare::compare_from_report;
use crate::study::portability::WasmPortabilityStatus;
use crate::subject::SubjectManifest;

use super::metrics::{update_confusion, Confusion, Label, Verdict};
use super::score::{label_for_scenario, scenario_from_name, score_case, ScenarioKind};
use super::suite::SuiteId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchCaseResult {
    pub subject: String,
    pub subject_toml: String,
    pub language: String,
    pub scenario: ScenarioKind,
    pub label: Label,
    pub wasm_status: WasmPortabilityStatus,
    pub verdict: Verdict,
    pub rationale: String,
    pub num_flows: usize,
    pub has_external_to_prompt_flow: bool,
    pub protocol_wins: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_path: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LanguageSupport {
    pub total: usize,
    pub scanned: usize,
    pub failed: usize,
    pub direct_wasm: usize,
    pub wasm_with_shim: usize,
    pub native_only: usize,
    pub unsupported: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportSummary {
    pub total_cases: usize,
    pub scanned_cases: usize,
    pub failed_cases: usize,
    pub scan_success_rate: f64,
    pub by_language: HashMap<String, LanguageSupport>,
    pub by_wasm_status: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionSummary {
    pub confusion: Confusion,
    pub precision: Option<f64>,
    pub recall: Option<f64>,
    pub f1: Option<f64>,
    pub specificity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationSummary {
    pub oracle_suite: String,
    pub protocol_vs_stdout_disagreements: usize,
    pub protocol_compare_errors: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchReport {
    pub run_id: String,
    pub suite: String,
    pub case_results: Vec<BenchCaseResult>,
    pub support: SupportSummary,
    pub detection: DetectionSummary,
    pub verification: VerificationSummary,
}

pub fn run_id() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("run-{secs}")
}

pub fn run_bench(
    manifest_dir: &Path,
    suite: SuiteId,
    subject_paths: &[PathBuf],
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
    max_output_bytes: usize,
    run_compare: bool,
    write_per_case_reports: bool,
    reports_dir: Option<&Path>,
) -> Result<BenchReport> {
    let run_id = run_id();
    let per_case_dir = reports_dir.map(|dir| dir.join("cases"));

    if let Some(dir) = per_case_dir.as_ref() {
        fs::create_dir_all(dir)
            .with_context(|| format!("create per-case report dir {}", dir.display()))?;
    }

    let mut case_results = Vec::new();
    let mut confusion = Confusion::default();
    let mut protocol_disagreements = 0usize;
    let mut protocol_compare_errors = 0usize;

    for subject_path in subject_paths {
        let case = bench_one_case(
            manifest_dir,
            subject_path,
            env,
            data_dir,
            max_output_bytes,
            run_compare,
            write_per_case_reports,
            per_case_dir.as_deref(),
        )?;

        if case.protocol_wins == Some(true) {
            protocol_disagreements += 1;
        }
        if case.scan_error.is_some() {
            protocol_compare_errors += 1;
        }

        if case.scan_error.is_none() {
            update_confusion(&mut confusion, case.label, case.verdict);
        } else {
            confusion.errors += 1;
        }
        case_results.push(case);
    }

    let support = support_summary(&case_results);
    let detection = DetectionSummary {
        precision: confusion.precision(),
        recall: confusion.recall(),
        f1: confusion.f1(),
        specificity: confusion.specificity(),
        confusion,
    };

    Ok(BenchReport {
        run_id,
        suite: suite.as_str().to_string(),
        case_results,
        support,
        detection,
        verification: VerificationSummary {
            oracle_suite: suite.as_str().to_string(),
            protocol_vs_stdout_disagreements: protocol_disagreements,
            protocol_compare_errors,
        },
    })
}

fn bench_one_case(
    manifest_dir: &Path,
    subject_path: &Path,
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
    max_output_bytes: usize,
    run_compare: bool,
    write_report: bool,
    reports_dir: Option<&Path>,
) -> Result<BenchCaseResult> {
    let raw = fs::read_to_string(subject_path)
        .with_context(|| format!("read subject {}", subject_path.display()))?;
    let subject: SubjectManifest = toml::from_str(&raw)
        .with_context(|| format!("parse subject {}", subject_path.display()))?;

    let scenario = scenario_from_name(&subject.name);
    let label = label_for_scenario(scenario);
    let language = format!("{:?}", subject.language);

    let scan_env = default_env_for_subject(&subject, env);
    let effective_data_dir = resolve_data_dir(manifest_dir, &subject, data_dir)?;

    match scan_subject(
        &subject,
        &scan_env,
        effective_data_dir.as_deref(),
        max_output_bytes,
    ) {
        Ok(result) => {
            let (verdict, rationale) = score_case(&result.report, scenario);
            let protocol_wins = if run_compare {
                let row = compare_from_report(&subject, subject_path, &result.report);
                Some(row.protocol_wins)
            } else {
                None
            };

            let report_path = if write_report {
                let dir = reports_dir.context("missing per-case report dir")?;
                let file = dir.join(format!("{}.json", subject.name));
                let json = serde_json::to_string_pretty(&result.report)?;
                fs::write(&file, json)
                    .with_context(|| format!("write report {}", file.display()))?;
                Some(file.to_string_lossy().into_owned())
            } else {
                None
            };

            Ok(BenchCaseResult {
                subject: subject.name.clone(),
                subject_toml: subject_path.to_string_lossy().into_owned(),
                language,
                scenario,
                label,
                wasm_status: result.adaptation_status.into(),
                verdict,
                rationale,
                num_flows: result.report.summary.num_flows,
                has_external_to_prompt_flow: result.report.summary.has_external_to_prompt_flow,
                protocol_wins,
                scan_error: None,
                report_path,
            })
        }
        Err(err) => Ok(BenchCaseResult {
            subject: subject.name,
            subject_toml: subject_path.to_string_lossy().into_owned(),
            language,
            scenario,
            label,
            wasm_status: WasmPortabilityStatus::Failed,
            verdict: Verdict::Error,
            rationale: err.to_string(),
            num_flows: 0,
            has_external_to_prompt_flow: false,
            protocol_wins: None,
            scan_error: Some(err.to_string()),
            report_path: None,
        }),
    }
}

fn support_summary(cases: &[BenchCaseResult]) -> SupportSummary {
    let total_cases = cases.len();
    let scanned_cases = cases.iter().filter(|c| c.scan_error.is_none()).count();
    let failed_cases = total_cases.saturating_sub(scanned_cases);
    let scan_success_rate = if total_cases == 0 {
        0.0
    } else {
        scanned_cases as f64 / total_cases as f64
    };

    let mut by_language: HashMap<String, LanguageSupport> = HashMap::new();
    let mut by_wasm_status: HashMap<String, usize> = HashMap::new();

    for case in cases {
        let entry = by_language.entry(case.language.clone()).or_default();
        entry.total += 1;
        if case.scan_error.is_some() {
            entry.failed += 1;
        } else {
            entry.scanned += 1;
        }
        match case.wasm_status {
            WasmPortabilityStatus::DirectWasm => entry.direct_wasm += 1,
            WasmPortabilityStatus::WasmWithShim => entry.wasm_with_shim += 1,
            WasmPortabilityStatus::NativeOnly => entry.native_only += 1,
            WasmPortabilityStatus::Unsupported => entry.unsupported += 1,
            WasmPortabilityStatus::Failed => {}
        }

        if case.scan_error.is_none() {
            let status = format!("{:?}", case.wasm_status);
            *by_wasm_status.entry(status).or_default() += 1;
        }
    }

    SupportSummary {
        total_cases,
        scanned_cases,
        failed_cases,
        scan_success_rate,
        by_language,
        by_wasm_status,
    }
}

pub fn write_bench_report(report: &BenchReport, out_dir: &Path) -> Result<()> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("create bench output dir {}", out_dir.display()))?;

    let json_path = out_dir.join("summary.json");
    fs::write(
        &json_path,
        serde_json::to_string_pretty(report).context("serialize bench summary json")?,
    )
    .with_context(|| format!("write {}", json_path.display()))?;

    let md_path = out_dir.join("summary.md");
    fs::write(&md_path, render_markdown(report))
        .with_context(|| format!("write {}", md_path.display()))?;

    if let Some(parent) = out_dir.parent() {
        let latest = parent.join("LATEST.txt");
        fs::write(
            &latest,
            format!(
                "run_id={}\nsuite={}\npath={}\n",
                report.run_id,
                report.suite,
                out_dir.display()
            ),
        )
        .with_context(|| format!("write {}", latest.display()))?;
    }

    Ok(())
}

fn render_markdown(report: &BenchReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Bench Report: {}\n\n", report.run_id));
    out.push_str(&format!("- **Suite**: `{}`\n", report.suite));
    out.push_str(&format!(
        "- **Scan success rate**: {:.1}% ({}/{})\n",
        report.support.scan_success_rate * 100.0,
        report.support.scanned_cases,
        report.support.total_cases
    ));

    let d = &report.detection.confusion;
    out.push_str("\n## Detection (oracle)\n\n");
    out.push_str("| Metric | Value |\n|--------|-------|\n");
    out.push_str(&format!("| TP | {} |\n", d.tp));
    out.push_str(&format!("| FN | {} |\n", d.fn_count));
    out.push_str(&format!("| FP | {} |\n", d.fp));
    out.push_str(&format!("| TN | {} |\n", d.tn));
    out.push_str(&format!("| Errors | {} |\n", d.errors));
    if let Some(p) = report.detection.precision {
        out.push_str(&format!("| Precision | {:.3} |\n", p));
    }
    if let Some(r) = report.detection.recall {
        out.push_str(&format!("| Recall | {:.3} |\n", r));
    }
    if let Some(f1) = report.detection.f1 {
        out.push_str(&format!("| F1 | {:.3} |\n", f1));
    }

    out.push_str("\n## Support by language\n\n");
    out.push_str("| Language | Total | Scanned | Failed | DirectWasm | WasmShim | NativeOnly |\n");
    out.push_str("|----------|-------|---------|--------|------------|----------|------------|\n");
    let mut langs: Vec<_> = report.support.by_language.iter().collect();
    langs.sort_by(|a, b| a.0.cmp(b.0));
    for (lang, stats) in langs {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            lang,
            stats.total,
            stats.scanned,
            stats.failed,
            stats.direct_wasm,
            stats.wasm_with_shim,
            stats.native_only
        ));
    }

    out.push_str("\n## Verification\n\n");
    out.push_str(&format!(
        "- Protocol vs stdout disagreements: {}\n",
        report.verification.protocol_vs_stdout_disagreements
    ));
    out.push_str(&format!(
        "- Scan errors: {}\n",
        report.verification.protocol_compare_errors
    ));

    out.push_str("\n## Cases\n\n");
    out.push_str("| Subject | Scenario | WASM | Verdict | Flows | Rationale |\n");
    out.push_str("|---------|----------|------|---------|-------|----------|\n");
    for case in &report.case_results {
        let rationale = case
            .scan_error
            .as_deref()
            .unwrap_or(case.rationale.as_str());
        out.push_str(&format!(
            "| {} | {:?} | {:?} | {:?} | {} | {} |\n",
            case.subject,
            case.scenario,
            case.wasm_status,
            case.verdict,
            case.num_flows,
            rationale.replace('|', "\\|")
        ));
    }

    out
}
