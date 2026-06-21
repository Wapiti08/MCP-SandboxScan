use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use walkdir::{DirEntry, WalkDir};

use crate::mcp::transcript::{McpDirection, McpTranscript};
use crate::monitor::event::MonitorEventKind;
use crate::scan::report::ScanReport;
use crate::subject::SubjectManifest;

use super::model::{CorpusFile, CorpusScanReport, RepoEntry, ToolSemanticProfile};
use super::tier::classify_tier;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SemanticCorpusReport {
    pub total_repos: usize,
    pub repos_with_tool_metadata: usize,
    pub repos_with_any_signal: usize,
    pub total_tools: usize,
    pub described_tools: usize,
    pub sensitive_tools: usize,
    #[serde(default)]
    pub by_capability: HashMap<String, usize>,
    #[serde(default)]
    pub by_source: HashMap<String, SemanticSourceStats>,
    pub cases: Vec<SemanticCorpusCase>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SemanticSourceStats {
    pub repos: usize,
    pub tools: usize,
    pub described_tools: usize,
    pub sensitive_tools: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCorpusCase {
    pub repo_id: String,
    pub tier: String,
    pub source: String,
    pub tool_count: usize,
    pub described_tools: usize,
    pub sensitive_tools: usize,
    #[serde(default)]
    pub by_capability: HashMap<String, usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SemanticCrossValidationReport {
    pub dynamic_cases: usize,
    pub cases_with_called_tool_metadata: usize,
    pub declared_egress_risk: usize,
    pub observed_network_egress: usize,
    pub declared_and_observed: usize,
    pub declared_only: usize,
    pub observed_only: usize,
    pub neither: usize,
    pub cases: Vec<SemanticCrossValidationCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCrossValidationCase {
    pub repo_id: String,
    pub called_tool: Option<String>,
    pub declared_capabilities: Vec<String>,
    pub declared_egress_risk: bool,
    pub observed_network_egress: bool,
    pub network_events: usize,
    pub relation: String,
}

pub fn profile_scan_report(report: &ScanReport) -> Option<ToolSemanticProfile> {
    let transcript = report.mcp_transcript.as_ref()?;
    profile_transcript(transcript)
}

pub fn build_semantic_corpus_report(
    corpus: &CorpusFile,
    scan_report: Option<&CorpusScanReport>,
    manifest_dir: &Path,
) -> SemanticCorpusReport {
    let dynamic = dynamic_profiles(scan_report);
    let mut report = SemanticCorpusReport {
        total_repos: corpus.repos.len(),
        ..SemanticCorpusReport::default()
    };

    for repo in &corpus.repos {
        let tier = if repo.tier.is_empty() {
            classify_tier(repo).to_string()
        } else {
            repo.tier.clone()
        };

        let (source, profile) = if let Some(profile) = dynamic.get(&repo.id) {
            ("dynamic_tools_list".to_string(), Some(profile.clone()))
        } else if let Some(root) = source_root(repo, manifest_dir) {
            if let Some(profile) = static_profile_repo(&root) {
                ("static_source".to_string(), Some(profile))
            } else if repo_semantic_signal(repo, &root) {
                ("repo_signal".to_string(), None)
            } else {
                ("none".to_string(), None)
            }
        } else {
            ("none".to_string(), None)
        };

        let case = SemanticCorpusCase {
            repo_id: repo.id.clone(),
            tier,
            source,
            tool_count: profile.as_ref().map(|p| p.tool_count).unwrap_or(0),
            described_tools: profile.as_ref().map(|p| p.described_tools).unwrap_or(0),
            sensitive_tools: profile.as_ref().map(|p| p.sensitive_tools).unwrap_or(0),
            by_capability: profile
                .as_ref()
                .map(|p| p.by_capability.clone())
                .unwrap_or_default(),
        };
        add_case_to_semantic_report(&mut report, case);
    }

    report
}

pub fn write_semantic_corpus_report(report: &SemanticCorpusReport, out_dir: &Path) -> Result<()> {
    fs::create_dir_all(out_dir)?;
    fs::write(
        out_dir.join("semantic_summary.json"),
        serde_json::to_string_pretty(report)?,
    )?;
    fs::write(
        out_dir.join("semantic_summary.md"),
        render_semantic_md(report),
    )?;
    fs::write(
        out_dir.join("semantic_tables.tex"),
        render_semantic_latex(report),
    )?;
    Ok(())
}

pub fn build_semantic_cross_validation_report(
    scan_report: &CorpusScanReport,
    out_dir: &Path,
) -> Result<SemanticCrossValidationReport> {
    let mut report = SemanticCrossValidationReport::default();

    for case in scan_report.cases.iter().filter(|case| case.scan_ok) {
        report.dynamic_cases += 1;
        let Some(path) =
            resolve_case_report_path(case.report_path.as_deref(), &case.repo_id, out_dir)
        else {
            continue;
        };
        let raw = fs::read_to_string(&path)?;
        let scan: ScanReport = serde_json::from_str(&raw)?;
        let row = cross_validate_case(&case.repo_id, &scan);

        if row.called_tool.is_some() && !row.declared_capabilities.is_empty() {
            report.cases_with_called_tool_metadata += 1;
        }
        if row.declared_egress_risk {
            report.declared_egress_risk += 1;
        }
        if row.observed_network_egress {
            report.observed_network_egress += 1;
        }
        match (row.declared_egress_risk, row.observed_network_egress) {
            (true, true) => report.declared_and_observed += 1,
            (true, false) => report.declared_only += 1,
            (false, true) => report.observed_only += 1,
            (false, false) => report.neither += 1,
        }
        report.cases.push(row);
    }

    Ok(report)
}

pub fn write_semantic_cross_validation_report(
    report: &SemanticCrossValidationReport,
    out_dir: &Path,
) -> Result<()> {
    fs::create_dir_all(out_dir)?;
    fs::write(
        out_dir.join("semantic_cross_validation.json"),
        serde_json::to_string_pretty(report)?,
    )?;
    fs::write(
        out_dir.join("semantic_cross_validation.md"),
        render_cross_validation_md(report),
    )?;
    fs::write(
        out_dir.join("semantic_cross_validation.tex"),
        render_cross_validation_latex(report),
    )?;
    Ok(())
}

pub fn profile_transcript(transcript: &McpTranscript) -> Option<ToolSemanticProfile> {
    let tools = first_tools_list(transcript)?;
    if tools.is_empty() {
        return None;
    }

    let mut profile = ToolSemanticProfile {
        tool_count: tools.len(),
        described_tools: 0,
        sensitive_tools: 0,
        by_capability: HashMap::new(),
    };

    for tool in tools {
        if tool
            .get("description")
            .and_then(Value::as_str)
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
        {
            profile.described_tools += 1;
        }

        let caps = classify_tool(tool);
        if caps.iter().any(|cap| *cap != "unknown") {
            profile.sensitive_tools += 1;
        }
        for cap in caps {
            *profile.by_capability.entry(cap.to_string()).or_default() += 1;
        }
    }

    Some(profile)
}

fn dynamic_profiles(
    scan_report: Option<&CorpusScanReport>,
) -> HashMap<String, ToolSemanticProfile> {
    let mut out = HashMap::new();
    let Some(scan_report) = scan_report else {
        return out;
    };
    for case in &scan_report.cases {
        if let Some(profile) = &case.tool_profile {
            out.insert(case.repo_id.clone(), profile.clone());
        }
    }
    out
}

fn resolve_case_report_path(
    report_path: Option<&str>,
    repo_id: &str,
    out_dir: &Path,
) -> Option<PathBuf> {
    if let Some(report_path) = report_path {
        let direct = PathBuf::from(report_path);
        if direct.exists() {
            return Some(direct);
        }
    }
    let slug = repo_id.replace('/', "__");
    let local = out_dir.join("cases").join(format!("{slug}.json"));
    local.exists().then_some(local)
}

fn cross_validate_case(repo_id: &str, report: &ScanReport) -> SemanticCrossValidationCase {
    let called_tool = called_tool_name(report);
    let declared_capabilities = called_tool
        .as_deref()
        .and_then(|name| called_tool_value(report, name))
        .map(|tool| {
            classify_tool(tool)
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let declared_egress_risk = declared_capabilities.iter().any(|cap| {
        matches!(
            cap.as_str(),
            "network" | "browser" | "cloud_saas" | "code_repo" | "database"
        )
    });
    let network_events = network_event_count(report);
    let observed_network_egress = network_events > 0;
    let relation = match (declared_egress_risk, observed_network_egress) {
        (true, true) => "declared_and_observed",
        (true, false) => "declared_only",
        (false, true) => "observed_only",
        (false, false) => "neither",
    }
    .to_string();

    SemanticCrossValidationCase {
        repo_id: repo_id.to_string(),
        called_tool,
        declared_capabilities,
        declared_egress_risk,
        observed_network_egress,
        network_events,
        relation,
    }
}

fn called_tool_name(report: &ScanReport) -> Option<String> {
    for event in &report.events {
        if event.kind == MonitorEventKind::McpToolCall {
            if let Some(tool) = event
                .evidence
                .pointer("/params/name")
                .and_then(Value::as_str)
            {
                return Some(tool.to_string());
            }
        }
    }
    None
}

fn called_tool_value<'a>(report: &'a ScanReport, called_tool: &str) -> Option<&'a Value> {
    let transcript = report.mcp_transcript.as_ref()?;
    let tools = first_tools_list(transcript)?;
    tools
        .iter()
        .find(|tool| tool.get("name").and_then(Value::as_str) == Some(called_tool))
}

fn network_event_count(report: &ScanReport) -> usize {
    report
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
        .count()
}

fn add_case_to_semantic_report(report: &mut SemanticCorpusReport, case: SemanticCorpusCase) {
    if case.source != "none" {
        report.repos_with_any_signal += 1;
    }

    let source_stats = report.by_source.entry(case.source.clone()).or_default();
    source_stats.repos += 1;

    if case.tool_count > 0 {
        report.repos_with_tool_metadata += 1;
        report.total_tools += case.tool_count;
        report.described_tools += case.described_tools;
        report.sensitive_tools += case.sensitive_tools;

        source_stats.tools += case.tool_count;
        source_stats.described_tools += case.described_tools;
        source_stats.sensitive_tools += case.sensitive_tools;

        for (cap, count) in &case.by_capability {
            *report.by_capability.entry(cap.clone()).or_default() += count;
        }
    }

    report.cases.push(case);
}

fn source_root(repo: &RepoEntry, manifest_dir: &Path) -> Option<PathBuf> {
    if let Some(toml) = &repo.subject_toml {
        let path = manifest_dir.join(toml);
        if let Ok(raw) = fs::read_to_string(path) {
            if let Ok(subject) = toml::from_str::<SubjectManifest>(&raw) {
                let source = if subject.source_dir.is_absolute() {
                    subject.source_dir
                } else {
                    manifest_dir.join(subject.source_dir)
                };
                if source.exists() {
                    return Some(source);
                }
            }
        }
    }

    let clone = manifest_dir
        .join("corpus")
        .join("clones")
        .join(repo.id.replace('/', "__"));
    clone.exists().then_some(clone)
}

fn static_profile_repo(root: &Path) -> Option<ToolSemanticProfile> {
    let mut seen = HashSet::new();
    let mut chunks = Vec::new();

    let walker = WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| should_visit(entry));

    for entry in walker.flatten() {
        if !entry.file_type().is_file() || !is_source_file(entry.path()) {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.len() > 1_500_000 {
            continue;
        }
        let Ok(raw) = fs::read_to_string(entry.path()) else {
            continue;
        };
        for chunk in static_tool_chunks(entry.path(), &raw) {
            let key = normalize_fingerprint(&chunk);
            if !key.is_empty() && seen.insert(key) {
                chunks.push(chunk);
            }
        }
    }

    if chunks.is_empty() {
        return None;
    }

    let mut profile = ToolSemanticProfile {
        tool_count: chunks.len(),
        described_tools: 0,
        sensitive_tools: 0,
        by_capability: HashMap::new(),
    };

    for chunk in chunks {
        if chunk.to_lowercase().contains("description") || looks_like_docstring(&chunk) {
            profile.described_tools += 1;
        }
        let caps = classify_text_to_caps(&chunk);
        if caps.iter().any(|cap| *cap != "unknown") {
            profile.sensitive_tools += 1;
        }
        for cap in caps {
            *profile.by_capability.entry(cap.to_string()).or_default() += 1;
        }
    }

    Some(profile)
}

fn should_visit(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    !matches!(
        name.as_ref(),
        ".git"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | ".venv"
            | "venv"
            | "__pycache__"
            | "vendor"
            | ".next"
            | "coverage"
    )
}

fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "py" | "go" | "rs")
    )
}

fn static_tool_chunks(path: &Path, raw: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let path_text = path.to_string_lossy().to_lowercase();
    if path_text.contains("/test")
        || path_text.contains("/tests")
        || path_text.contains("__tests__")
        || path_text.contains(".spec.")
        || path_text.contains(".test.")
    {
        return chunks;
    }

    let lines: Vec<&str> = raw.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        if is_tool_registration_line(&lower) {
            chunks.push(window(&lines, idx, 4, 80));
        }
    }
    chunks
}

fn is_tool_registration_line(lower: &str) -> bool {
    lower.contains("server.tool(")
        || lower.contains("mcp.tool(")
        || lower.contains("mcp.newtool(")
        || lower.contains("newtool(")
        || lower.contains("add_tool(")
        || lower.contains("@mcp.tool")
        || lower.contains("listtoolsrequestschema")
}

fn window(lines: &[&str], idx: usize, before: usize, after: usize) -> String {
    let start = idx.saturating_sub(before);
    let end = (idx + after).min(lines.len().saturating_sub(1));
    lines[start..=end].join("\n")
}

fn normalize_fingerprint(text: &str) -> String {
    text.split_whitespace()
        .take(120)
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn looks_like_docstring(text: &str) -> bool {
    text.contains("\"\"\"") || text.contains("'''") || text.contains("/**")
}

fn repo_semantic_signal(repo: &RepoEntry, root: &Path) -> bool {
    if semantic_signal_text(repo).is_some() {
        return true;
    }
    for name in ["README.md", "README.rst", "readme.md"] {
        let path = root.join(name);
        let Ok(raw) = fs::read_to_string(path) else {
            continue;
        };
        if classify_text_to_caps(&raw)
            .iter()
            .any(|cap| *cap != "unknown")
        {
            return true;
        }
    }
    false
}

fn semantic_signal_text(repo: &RepoEntry) -> Option<String> {
    let mut text = format!("{} {}", repo.id, repo.language.as_deref().unwrap_or(""));
    for topic in &repo.topics {
        text.push(' ');
        text.push_str(topic);
    }
    classify_text_to_caps(&text)
        .iter()
        .any(|cap| *cap != "unknown")
        .then_some(text)
}

fn first_tools_list(transcript: &McpTranscript) -> Option<&Vec<Value>> {
    for event in &transcript.events {
        if !matches!(event.direction, McpDirection::ServerToClient) {
            continue;
        }
        let Some(tools) = event
            .payload
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(Value::as_array)
        else {
            continue;
        };
        return Some(tools);
    }
    None
}

fn classify_tool(tool: &Value) -> Vec<&'static str> {
    let text = tool_text(tool);
    classify_text_to_caps(&text)
}

fn classify_text_to_caps(raw: &str) -> Vec<&'static str> {
    let text = raw.to_lowercase();
    let mut caps = Vec::new();

    for (capability, keywords) in CAPABILITY_KEYWORDS {
        if keywords.iter().any(|keyword| text.contains(keyword)) {
            caps.push(*capability);
        }
    }

    if caps.is_empty() {
        caps.push("unknown");
    }
    caps
}

fn render_semantic_md(r: &SemanticCorpusReport) -> String {
    let mut out = format!(
        "# Semantic Corpus Scan\n\n\
         - Repos: {}\n\
         - Repos with tool metadata: {}\n\
         - Repos with any semantic signal: {}\n\
         - Tools: {}\n\
         - Described tools: {}\n\
         - Sensitive tools: {}\n\n",
        r.total_repos,
        r.repos_with_tool_metadata,
        r.repos_with_any_signal,
        r.total_tools,
        r.described_tools,
        r.sensitive_tools,
    );

    out.push_str("## By extraction source\n\n");
    out.push_str("| Source | Repos | Tools | Described | Sensitive |\n");
    out.push_str("|--------|-------|-------|-----------|-----------|\n");
    for source in ["dynamic_tools_list", "static_source", "repo_signal", "none"] {
        if let Some(stats) = r.by_source.get(source) {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                source, stats.repos, stats.tools, stats.described_tools, stats.sensitive_tools
            ));
        }
    }

    out.push_str("\n## Tool capabilities\n\n");
    out.push_str("| Capability | Tools |\n");
    out.push_str("|------------|-------|\n");
    let mut caps: Vec<_> = r.by_capability.iter().collect();
    caps.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    for (cap, count) in caps {
        out.push_str(&format!("| {} | {} |\n", cap, count));
    }

    out.push_str("\n## Cases\n\n");
    out.push_str("| Repo | Tier | Source | Tools | Sensitive |\n");
    out.push_str("|------|------|--------|-------|-----------|\n");
    for case in &r.cases {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            case.repo_id, case.tier, case.source, case.tool_count, case.sensitive_tools
        ));
    }
    out
}

fn render_semantic_latex(r: &SemanticCorpusReport) -> String {
    let sensitive_rate = if r.total_tools == 0 {
        0.0
    } else {
        r.sensitive_tools as f64 / r.total_tools as f64 * 100.0
    };
    let mut caps: Vec<_> = r.by_capability.iter().collect();
    caps.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

    let mut out = format!(
        "% Semantic-only corpus evaluation tables.\n\n\
         \\begin{{table}}[t]\n\
         \\centering\n\
         \\caption{{Semantic extraction coverage over the full MCP corpus.}}\n\
         \\label{{tab:semantic-full-coverage}}\n\
         \\begin{{tabular}}{{lrrrr}}\n\
         \\toprule\n\
         Source & Repos & Tools & Described & Sensitive \\\\\n\
         \\midrule\n"
    );
    for source in ["dynamic_tools_list", "static_source", "repo_signal", "none"] {
        if let Some(stats) = r.by_source.get(source) {
            out.push_str(&format!(
                "{} & {} & {} & {} & {} \\\\\n",
                latex_source_label(source),
                stats.repos,
                stats.tools,
                stats.described_tools,
                stats.sensitive_tools
            ));
        }
    }
    out.push_str(&format!(
        "\\midrule\n\
         Total tool metadata & {} & {} & {} & {} ({:.1}\\%) \\\\\n\
         \\bottomrule\n\
         \\end{{tabular}}\n\
         \\end{{table}}\n\n",
        r.repos_with_tool_metadata,
        r.total_tools,
        r.described_tools,
        r.sensitive_tools,
        sensitive_rate
    ));

    out.push_str(
        "\\begin{table}[t]\n\
         \\centering\n\
         \\caption{Semantic capability distribution across extracted MCP tools.}\n\
         \\label{tab:semantic-full-capabilities}\n\
         \\begin{tabular}{lr}\n\
         \\toprule\n\
         Capability & Tools \\\\\n\
         \\midrule\n",
    );
    for (cap, count) in caps {
        out.push_str(&format!(
            "{} & {} \\\\\n",
            latex_capability_label(cap),
            count
        ));
    }
    out.push_str(
        "\\bottomrule\n\
         \\end{tabular}\n\
         \\end{table}\n",
    );
    out
}

fn render_cross_validation_md(r: &SemanticCrossValidationReport) -> String {
    let mut out = format!(
        "# Semantic Cross-Validation\n\n\
         - Dynamic cases: {}\n\
         - Cases with called-tool metadata: {}\n\
         - Declared egress-risk tools: {}\n\
         - Observed network egress: {}\n\
         - Declared and observed: {}\n\
         - Declared only: {}\n\
         - Observed only: {}\n\
         - Neither: {}\n\n",
        r.dynamic_cases,
        r.cases_with_called_tool_metadata,
        r.declared_egress_risk,
        r.observed_network_egress,
        r.declared_and_observed,
        r.declared_only,
        r.observed_only,
        r.neither,
    );

    out.push_str("| Relation | Count |\n");
    out.push_str("|----------|-------|\n");
    out.push_str(&format!(
        "| Declared and observed | {} |\n",
        r.declared_and_observed
    ));
    out.push_str(&format!("| Declared only | {} |\n", r.declared_only));
    out.push_str(&format!("| Observed only | {} |\n", r.observed_only));
    out.push_str(&format!("| Neither | {} |\n", r.neither));

    out.push_str("\n## Cases\n\n");
    out.push_str("| Repo | Called tool | Declared capabilities | Network events | Relation |\n");
    out.push_str("|------|-------------|-----------------------|----------------|----------|\n");
    for case in &r.cases {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            case.repo_id,
            case.called_tool.as_deref().unwrap_or("-"),
            case.declared_capabilities.join(", "),
            case.network_events,
            case.relation
        ));
    }
    out
}

fn render_cross_validation_latex(r: &SemanticCrossValidationReport) -> String {
    format!(
        "\\begin{{table}}[t]\n\
         \\centering\n\
         \\caption{{Cross-validation between declared tool semantics and runtime network evidence.}}\n\
         \\label{{tab:semantic-cross-validation}}\n\
         \\begin{{tabular}}{{lrp{{0.45\\linewidth}}}}\n\
         \\toprule\n\
         Relation & Repos & Interpretation \\\\\n\
         \\midrule\n\
         Declared egress risk and observed egress & {} & Confirmed by runtime evidence \\\\\n\
         Declared egress risk only & {} & Declared capability was not exercised by this run \\\\\n\
         Observed egress only & {} & Potential metadata understatement or classifier miss \\\\\n\
         Neither declared nor observed & {} & No egress evidence in this run \\\\\n\
         \\bottomrule\n\
         \\end{{tabular}}\n\
         \\end{{table}}\n",
        r.declared_and_observed, r.declared_only, r.observed_only, r.neither,
    )
}

fn latex_source_label(source: &str) -> &'static str {
    match source {
        "dynamic_tools_list" => "Dynamic \\texttt{tools/list}",
        "static_source" => "Static source registration",
        "repo_signal" => "Repo-level signal only",
        "none" => "No semantic signal",
        _ => "Other",
    }
}

fn latex_capability_label(cap: &str) -> &'static str {
    match cap {
        "network" => "Network access",
        "browser" => "Browser automation",
        "cloud_saas" => "Cloud or SaaS operation",
        "database" => "Database access",
        "code_repo" => "Code repository operation",
        "filesystem" => "Filesystem access",
        "credential" => "Credential or secret handling",
        "shell" => "Shell or process execution",
        "unknown" => "Unknown / benign utility",
        _ => "Other",
    }
}

fn tool_text(tool: &Value) -> String {
    let mut text = String::new();
    append_string_field(tool, "name", &mut text);
    append_string_field(tool, "description", &mut text);
    append_schema_text(tool.get("inputSchema"), &mut text);
    append_schema_text(tool.get("input_schema"), &mut text);
    text.to_lowercase()
}

fn append_string_field(value: &Value, field: &str, out: &mut String) {
    if let Some(s) = value.get(field).and_then(Value::as_str) {
        out.push(' ');
        out.push_str(s);
    }
}

fn append_schema_text(value: Option<&Value>, out: &mut String) {
    let Some(value) = value else {
        return;
    };
    match value {
        Value::String(s) => {
            out.push(' ');
            out.push_str(s);
        }
        Value::Array(items) => {
            for item in items {
                append_schema_text(Some(item), out);
            }
        }
        Value::Object(map) => {
            for (key, value) in map {
                if matches!(
                    key.as_str(),
                    "$schema" | "type" | "required" | "additionalProperties"
                ) {
                    continue;
                }
                out.push(' ');
                out.push_str(key);
                append_schema_text(Some(value), out);
            }
        }
        _ => {}
    }
}

const CAPABILITY_KEYWORDS: &[(&str, &[&str])] = &[
    (
        "shell",
        &[
            "execute command",
            "run command",
            "shell",
            "terminal",
            "subprocess",
            "powershell",
            "bash",
            "process",
        ],
    ),
    (
        "filesystem",
        &[
            "filesystem",
            "file system",
            "read file",
            "write file",
            "directory",
            "folder",
            "absolute path",
            "local file",
        ],
    ),
    (
        "network",
        &[
            "http", "url", "request", "fetch", "scrape", "crawl", "webhook", "endpoint", "website",
        ],
    ),
    (
        "browser",
        &[
            "browser",
            "playwright",
            "chrome",
            "screenshot",
            "tab",
            "navigate",
            "click",
            "page",
        ],
    ),
    (
        "database",
        &[
            "database", "sql", "query", "mysql", "postgres", "mongodb", "redis", "qdrant", "vector",
        ],
    ),
    (
        "credential",
        &[
            "token",
            "secret",
            "credential",
            "password",
            "api key",
            "apikey",
            "oauth",
            "private key",
        ],
    ),
    (
        "code_repo",
        &[
            "github",
            "gitlab",
            "repository",
            "repo",
            "pull request",
            "commit",
            "branch",
            "issue",
        ],
    ),
    (
        "cloud_saas",
        &[
            "aws",
            "azure",
            "gcp",
            "kubernetes",
            "k8s",
            "docker",
            "slack",
            "jira",
            "confluence",
            "figma",
            "notion",
            "gmail",
            "salesforce",
            "atlassian",
            "linear",
        ],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn classifies_sensitive_tool_from_description_and_schema() {
        let tool = json!({
            "name": "run_shell",
            "description": "Run command in a terminal",
            "inputSchema": {
                "properties": {
                    "cmd": {
                        "description": "Shell command to execute",
                        "type": "string"
                    }
                }
            }
        });

        let caps = classify_tool(&tool);
        assert!(caps.contains(&"shell"));
    }

    #[test]
    fn unknown_when_no_keywords_match() {
        let tool = json!({
            "name": "echo",
            "description": "Echo text",
            "inputSchema": {"type": "object"}
        });

        assert_eq!(classify_tool(&tool), vec!["unknown"]);
    }
}
