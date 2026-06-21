use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};

use crate::eval::run_id;
use crate::pipeline::case_study::{default_env_for_subject, resolve_data_dir};
use crate::pipeline::{ScanLimits, scan_subject_with_limits};
use crate::scan::report::ScanReport;
use crate::subject::SubjectManifest;

use super::model::{
    ClassStats, CorpusFile, CorpusScanCase, CorpusScanReport, LatencyStats, RepoEntry, TierStats,
    ToolSemanticAggregate, ToolSemanticSummary,
};
use super::semantic::profile_scan_report;
use super::tier::{assign_tiers, classify_tier};

pub struct ScanOptions {
    pub manifest_dir: PathBuf,
    pub out_dir: PathBuf,
    pub env: HashMap<String, String>,
    pub max_output_bytes: usize,
    pub limits: ScanLimits,
}

pub fn run_corpus_scan(corpus: &mut CorpusFile, opts: &ScanOptions) -> Result<CorpusScanReport> {
    assign_tiers(&mut corpus.repos);
    fs::create_dir_all(&opts.out_dir)?;
    let cases_dir = opts.out_dir.join("cases");
    fs::create_dir_all(&cases_dir)?;

    let mut cases = Vec::new();
    let mut by_class: HashMap<String, ClassStats> = HashMap::new();
    let mut by_ecosystem: HashMap<String, ClassStats> = HashMap::new();
    let mut by_failure_category: HashMap<String, usize> = HashMap::new();

    let total = corpus
        .repos
        .iter()
        .filter(|r| r.subject_toml.is_some())
        .count();
    let mut position = 0usize;

    for repo in corpus.repos.iter_mut() {
        let Some(toml) = repo.subject_toml.clone() else {
            continue;
        };

        position += 1;
        eprintln!("[{position}/{total}] scanning {} ...", repo.id);

        let case = scan_one_repo(repo, &toml, opts, &cases_dir);
        let scan_ok = case.scan_ok;
        let stats = by_class.entry(repo.wasm_class.clone()).or_default();
        stats.total += 1;

        let eco_key = if case.ecosystem.is_empty() {
            "unknown".to_string()
        } else {
            case.ecosystem.clone()
        };
        let eco_stats = by_ecosystem.entry(eco_key).or_default();
        eco_stats.total += 1;

        if scan_ok {
            repo.scan_status = "scan_ok".into();
            stats.scanned += 1;
            eco_stats.scanned += 1;
            if case.has_flow {
                stats.suspicious += 1;
                eco_stats.suspicious += 1;
            }
        } else {
            repo.scan_status = "scan_fail".into();
            if let Some(cat) = &case.failure_category {
                *by_failure_category.entry(cat.clone()).or_default() += 1;
            }
        }

        cases.push(case);
        let status = if scan_ok { "ok" } else { "fail" };
        let latency = cases
            .last()
            .and_then(|c| c.total_ms)
            .map(|ms| format!(" {ms}ms"))
            .unwrap_or_default();
        eprintln!("[{position}/{total}] {} -> {status}{latency}", repo.id);
    }

    let total_repos = corpus.repos.len();
    let resolved_repos = corpus.repos.iter().filter(|r| r.resolved).count();
    let scanned_repos = cases.iter().filter(|c| c.scan_ok).count();
    let suspicious = cases.iter().filter(|c| c.has_flow).count();
    let tier1 = compute_tier_stats(&cases, "tier1");
    let semantic = compute_semantic_summary(&cases);

    Ok(CorpusScanReport {
        run_id: run_id(),
        total_repos,
        resolved_repos,
        scanned_repos,
        scan_success_rate: if resolved_repos == 0 {
            0.0
        } else {
            scanned_repos as f64 / resolved_repos as f64
        },
        suspicious_rate: if scanned_repos == 0 {
            0.0
        } else {
            suspicious as f64 / scanned_repos as f64
        },
        by_wasm_class: by_class,
        by_ecosystem,
        by_failure_category,
        latency: compute_latency_stats(&cases),
        tier1,
        semantic,
        cases,
    })
}

fn scan_one_repo(
    repo: &RepoEntry,
    toml_path: &str,
    opts: &ScanOptions,
    cases_dir: &Path,
) -> CorpusScanCase {
    let mut base = CorpusScanCase {
        repo_id: repo.id.clone(),
        subject_toml: toml_path.to_string(),
        language: repo.language.clone(),
        wasm_class: repo.wasm_class.clone(),
        wasm_status: "unknown".into(),
        scan_ok: false,
        has_flow: false,
        num_flows: 0,
        num_sinks: 0,
        error: None,
        report_path: None,
        failure_category: None,
        stars: repo.stars,
        dep_count: repo.dep_count,
        ecosystem: repo.ecosystem.clone(),
        tier: if repo.tier.is_empty() {
            classify_tier(repo).to_string()
        } else {
            repo.tier.clone()
        },
        build_ms: None,
        scan_ms: None,
        total_ms: None,
        tool_profile: None,
    };

    let path = PathBuf::from(toml_path);
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return fail(base, e.to_string()),
    };
    let subject: SubjectManifest = match toml::from_str(&raw) {
        Ok(s) => s,
        Err(e) => return fail(base, e.to_string()),
    };

    let env = default_env_for_subject(&subject, &opts.env);
    let data_dir = match resolve_data_dir(&opts.manifest_dir, &subject, None) {
        Ok(d) => d,
        Err(e) => return fail(base, e.to_string()),
    };

    let started = Instant::now();
    match scan_subject_with_limits(
        &subject,
        &env,
        data_dir.as_deref(),
        opts.max_output_bytes,
        opts.limits.clone(),
    ) {
        Ok(result) => {
            base.total_ms = Some(started.elapsed().as_millis());
            base.build_ms = Some(result.timing.build_ms);
            base.scan_ms = Some(result.timing.scan_ms);
            let slug = repo.id.replace('/', "__");
            let report_path = cases_dir.join(format!("{slug}.json"));
            if let Err(e) = fs::write(
                &report_path,
                serde_json::to_string_pretty(&result.report).unwrap_or_default(),
            ) {
                return fail(base, e.to_string());
            }
            base.wasm_status = format!("{:?}", result.adaptation_status);
            base.scan_ok = true;
            base.has_flow = result.report.summary.has_external_to_prompt_flow;
            base.num_flows = result.report.summary.num_flows;
            base.num_sinks = result.report.summary.num_sinks;
            base.tool_profile = profile_scan_report(&result.report);
            base.report_path = Some(report_path.to_string_lossy().into_owned());
            base
        }
        Err(e) => {
            base.total_ms = Some(started.elapsed().as_millis());
            fail(base, format!("{e:#}"))
        }
    }
}

pub fn enrich_corpus_report_from_path(summary_path: &Path) -> Result<CorpusScanReport> {
    let raw = fs::read_to_string(summary_path)
        .with_context(|| format!("read {}", summary_path.display()))?;
    let mut report: CorpusScanReport =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", summary_path.display()))?;
    let out_dir = summary_path
        .parent()
        .with_context(|| format!("missing parent directory for {}", summary_path.display()))?;
    enrich_corpus_report(&mut report, out_dir)?;
    Ok(report)
}

pub fn enrich_corpus_report(report: &mut CorpusScanReport, out_dir: &Path) -> Result<()> {
    for case in &mut report.cases {
        if !case.scan_ok {
            continue;
        }
        let Some(path) = resolve_case_report_path(case, out_dir) else {
            continue;
        };
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("read case report {}", path.display()))?;
        let scan_report: ScanReport = serde_json::from_str(&raw)
            .with_context(|| format!("parse case report {}", path.display()))?;
        case.tool_profile = profile_scan_report(&scan_report);
    }
    report.semantic = compute_semantic_summary(&report.cases);
    Ok(())
}

fn resolve_case_report_path(case: &CorpusScanCase, out_dir: &Path) -> Option<PathBuf> {
    if let Some(report_path) = &case.report_path {
        let direct = PathBuf::from(report_path);
        if direct.exists() {
            return Some(direct);
        }
    }

    let slug = case.repo_id.replace('/', "__");
    let local = out_dir.join("cases").join(format!("{slug}.json"));
    if local.exists() {
        return Some(local);
    }

    None
}

fn fail(mut case: CorpusScanCase, err: String) -> CorpusScanCase {
    case.failure_category = Some(classify_failure(&err).to_string());
    case.error = Some(err);
    case
}

fn classify_failure(err: &str) -> &'static str {
    let lower = err.to_lowercase();
    if lower.contains("build command exited") || lower.contains("command timed out") {
        "build"
    } else if lower.contains("timed out after") {
        "timeout"
    } else if lower.contains("tools/call") || lower.contains("tools/list") {
        "mcp_call"
    } else if lower.contains("failed to spawn")
        || lower.contains("closed stdout")
        || lower.contains("no such file or directory")
    {
        "mcp_start"
    } else if lower.contains("cannot be scanned") || lower.contains("failed to adapt") {
        "adapt"
    } else {
        "other"
    }
}

fn compute_tier_stats(cases: &[CorpusScanCase], tier: &str) -> TierStats {
    let resolved = cases.iter().filter(|c| c.tier == tier).count();
    let scanned = cases.iter().filter(|c| c.tier == tier && c.scan_ok).count();
    let suspicious = cases
        .iter()
        .filter(|c| c.tier == tier && c.has_flow)
        .count();
    let ok: Vec<CorpusScanCase> = cases
        .iter()
        .filter(|c| c.tier == tier && c.scan_ok)
        .cloned()
        .collect();

    TierStats {
        resolved,
        scanned,
        scan_success_rate: if resolved == 0 {
            0.0
        } else {
            scanned as f64 / resolved as f64
        },
        suspicious,
        suspicious_rate: if scanned == 0 {
            0.0
        } else {
            suspicious as f64 / scanned as f64
        },
        latency: compute_latency_stats(&ok),
    }
}

fn compute_latency_stats(cases: &[CorpusScanCase]) -> LatencyStats {
    let ok: Vec<_> = cases.iter().filter(|c| c.scan_ok).collect();
    if ok.is_empty() {
        return LatencyStats::default();
    }

    let mut build: Vec<u128> = ok.iter().filter_map(|c| c.build_ms).collect();
    let mut scan: Vec<u128> = ok.iter().filter_map(|c| c.scan_ms).collect();
    let mut total: Vec<u128> = ok.iter().filter_map(|c| c.total_ms).collect();
    build.sort_unstable();
    scan.sort_unstable();
    total.sort_unstable();

    LatencyStats {
        count: ok.len(),
        build_ms_p50: percentile(&build, 50),
        build_ms_p95: percentile(&build, 95),
        scan_ms_p50: percentile(&scan, 50),
        scan_ms_p95: percentile(&scan, 95),
        total_ms_p50: percentile(&total, 50),
        total_ms_p95: percentile(&total, 95),
    }
}

fn compute_semantic_summary(cases: &[CorpusScanCase]) -> ToolSemanticSummary {
    ToolSemanticSummary {
        all: aggregate_semantic(cases.iter()),
        tier1: aggregate_semantic(cases.iter().filter(|c| c.tier == "tier1")),
        tier2: aggregate_semantic(cases.iter().filter(|c| c.tier == "tier2")),
    }
}

fn aggregate_semantic<'a>(
    cases: impl Iterator<Item = &'a CorpusScanCase>,
) -> ToolSemanticAggregate {
    let mut out = ToolSemanticAggregate::default();
    for case in cases {
        let Some(profile) = &case.tool_profile else {
            continue;
        };
        out.repos_with_metadata += 1;
        out.total_tools += profile.tool_count;
        out.described_tools += profile.described_tools;
        out.sensitive_tools += profile.sensitive_tools;
        for (cap, count) in &profile.by_capability {
            *out.by_capability.entry(cap.clone()).or_default() += count;
        }
    }
    out
}

fn percentile(sorted: &[u128], pct: usize) -> u128 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = (sorted.len() * pct).div_ceil(100).saturating_sub(1);
    sorted[idx.min(sorted.len() - 1)]
}

pub fn write_corpus_report(report: &CorpusScanReport, out_dir: &Path) -> Result<()> {
    fs::write(
        out_dir.join("corpus_summary.json"),
        serde_json::to_string_pretty(report)?,
    )?;
    fs::write(out_dir.join("corpus_summary.md"), render_md(report))?;

    if let Some(parent) = out_dir.parent() {
        let latest = parent.join("CORPUS_LATEST.txt");
        fs::write(
            &latest,
            format!("run_id={}\npath={}\n", report.run_id, out_dir.display()),
        )?;
    }
    Ok(())
}

fn render_md(r: &CorpusScanReport) -> String {
    let mut out = format!(
        "# Corpus Scan: {}\n\n\
         - Repos: {}\n\
         - Resolved: {}\n\
         - Scanned: {}\n\
         - Scan success: {:.1}%\n\
         - Suspicious rate: {:.1}% (flows observed, NOT verified vulns)\n\n",
        r.run_id,
        r.total_repos,
        r.resolved_repos,
        r.scanned_repos,
        r.scan_success_rate * 100.0,
        r.suspicious_rate * 100.0,
    );

    out.push_str("## Tier-1 (dedicated MCP servers)\n\n");
    out.push_str(&format!(
        "- Resolved: {}\n\
         - Scanned: {}\n\
         - Scan success: {:.1}%\n\
         - Suspicious rate: {:.1}%\n\n",
        r.tier1.resolved,
        r.tier1.scanned,
        r.tier1.scan_success_rate * 100.0,
        r.tier1.suspicious_rate * 100.0,
    ));
    if r.tier1.latency.count > 0 {
        out.push_str(&format!(
            "- Total latency p50/p95: {}ms / {}ms\n\n",
            r.tier1.latency.total_ms_p50, r.tier1.latency.total_ms_p95
        ));
    }

    if r.latency.count > 0 {
        out.push_str("## Latency (successful scans)\n\n");
        out.push_str(&format!(
            "- Count: {}\n\
             - Build p50/p95: {}ms / {}ms\n\
             - Scan p50/p95: {}ms / {}ms\n\
             - Total p50/p95: {}ms / {}ms\n\n",
            r.latency.count,
            r.latency.build_ms_p50,
            r.latency.build_ms_p95,
            r.latency.scan_ms_p50,
            r.latency.scan_ms_p95,
            r.latency.total_ms_p50,
            r.latency.total_ms_p95,
        ));
    }

    if r.semantic.all.repos_with_metadata > 0 {
        out.push_str("## Tool metadata and semantic profile\n\n");
        out.push_str("| Subset | Repos with metadata | Tools | Described | Sensitive |\n");
        out.push_str("|--------|---------------------|-------|-----------|-----------|\n");
        push_semantic_row(&mut out, "All", &r.semantic.all);
        push_semantic_row(&mut out, "Tier-1", &r.semantic.tier1);
        push_semantic_row(&mut out, "Tier-2", &r.semantic.tier2);

        out.push_str("\n### Semantic capabilities\n\n");
        out.push_str("| Capability | Tools |\n");
        out.push_str("|------------|-------|\n");
        let mut caps: Vec<_> = r.semantic.all.by_capability.iter().collect();
        caps.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        for (cap, count) in caps {
            out.push_str(&format!("| {} | {} |\n", cap, count));
        }
        out.push('\n');
    }

    out.push_str("## By WASM class\n\n");
    out.push_str("| Class | Total | Scanned | Suspicious |\n");
    out.push_str("|-------|-------|---------|------------|\n");
    let mut classes: Vec<_> = r.by_wasm_class.iter().collect();
    classes.sort_by(|a, b| a.0.cmp(b.0));
    for (class, stats) in classes {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            class, stats.total, stats.scanned, stats.suspicious
        ));
    }

    out.push_str("\n## By ecosystem\n\n");
    out.push_str("| Ecosystem | Total | Scanned | Rate |\n");
    out.push_str("|-----------|-------|---------|------|\n");
    let mut ecosystems: Vec<_> = r.by_ecosystem.iter().collect();
    ecosystems.sort_by(|a, b| a.0.cmp(b.0));
    for (eco, stats) in ecosystems {
        let rate = if stats.total == 0 {
            0.0
        } else {
            stats.scanned as f64 / stats.total as f64 * 100.0
        };
        out.push_str(&format!(
            "| {} | {} | {} | {:.1}% |\n",
            eco, stats.total, stats.scanned, rate
        ));
    }

    if !r.by_failure_category.is_empty() {
        out.push_str("\n## Failure categories\n\n");
        out.push_str("| Category | Count |\n");
        out.push_str("|----------|-------|\n");
        let mut cats: Vec<_> = r.by_failure_category.iter().collect();
        cats.sort_by(|a, b| b.1.cmp(a.1));
        for (cat, count) in cats {
            out.push_str(&format!("| {} | {} |\n", cat, count));
        }
    }

    out.push_str("\n## Cases\n\n");
    out.push_str("| Repo | Tier | OK | Flows | Deps | Stars | Total ms | Error |\n");
    out.push_str("|------|------|----|-------|------|-------|----------|-------|\n");
    for case in &r.cases {
        let err = case.error.as_deref().unwrap_or("");
        let total_ms = case
            .total_ms
            .map(|ms| ms.to_string())
            .unwrap_or_else(|| "-".into());
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            case.repo_id,
            case.tier,
            case.scan_ok,
            case.num_flows,
            case.dep_count,
            case.stars,
            total_ms,
            err.replace('|', "\\|")
        ));
    }
    out
}

fn push_semantic_row(out: &mut String, label: &str, stats: &ToolSemanticAggregate) {
    out.push_str(&format!(
        "| {} | {} | {} | {} | {} |\n",
        label,
        stats.repos_with_metadata,
        stats.total_tools,
        stats.described_tools,
        stats.sensitive_tools,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_build_failure() {
        assert_eq!(
            classify_failure("build command exited with status exit status: 1"),
            "build"
        );
    }

    #[test]
    fn classifies_mcp_start_failure() {
        assert_eq!(
            classify_failure("MCP server closed stdout before JSON response"),
            "mcp_start"
        );
    }

    #[test]
    fn percentile_picks_median() {
        assert_eq!(percentile(&[10, 20, 30], 50), 20);
    }

    #[test]
    fn aggregates_semantic_profiles_by_tier() {
        let cases = vec![
            CorpusScanCase {
                repo_id: "a/b".into(),
                subject_toml: "a.toml".into(),
                language: None,
                wasm_class: "wasm-hard".into(),
                wasm_status: "NativeOnly".into(),
                scan_ok: true,
                has_flow: false,
                num_flows: 0,
                num_sinks: 0,
                error: None,
                report_path: None,
                failure_category: None,
                stars: 0,
                dep_count: 0,
                ecosystem: "npm".into(),
                tier: "tier1".into(),
                build_ms: None,
                scan_ms: None,
                total_ms: None,
                tool_profile: Some(crate::corpus::model::ToolSemanticProfile {
                    tool_count: 2,
                    described_tools: 2,
                    sensitive_tools: 1,
                    by_capability: HashMap::from([("shell".to_string(), 1)]),
                }),
            },
            CorpusScanCase {
                repo_id: "c/d".into(),
                subject_toml: "c.toml".into(),
                language: None,
                wasm_class: "wasm-hard".into(),
                wasm_status: "NativeOnly".into(),
                scan_ok: true,
                has_flow: false,
                num_flows: 0,
                num_sinks: 0,
                error: None,
                report_path: None,
                failure_category: None,
                stars: 0,
                dep_count: 0,
                ecosystem: "npm".into(),
                tier: "tier2".into(),
                build_ms: None,
                scan_ms: None,
                total_ms: None,
                tool_profile: None,
            },
        ];

        let summary = compute_semantic_summary(&cases);
        assert_eq!(summary.all.repos_with_metadata, 1);
        assert_eq!(summary.tier1.total_tools, 2);
        assert_eq!(summary.tier2.total_tools, 0);
    }
}
