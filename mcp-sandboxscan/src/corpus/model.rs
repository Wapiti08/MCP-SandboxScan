use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusFile {
    pub collected_at: String,
    #[serde(default)]
    pub queries: Vec<String>,
    pub repos: Vec<RepoEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoEntry {
    pub id: String,
    pub url: String,
    pub clone_url: String,
    #[serde(default)]
    pub stars: u64,
    pub language: Option<String>,
    #[serde(default)]
    pub topics: Vec<String>,
    pub wasm_class: String,
    #[serde(default)]
    pub resolved: bool,
    #[serde(default)]
    pub scan_status: String, // pending | resolved | scan_ok | scan_fail | skipped
    #[serde(default)]
    pub ecosystem: String,
    #[serde(default)]
    pub dep_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_toml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusScanCase {
    pub repo_id: String,
    pub subject_toml: String,
    pub language: Option<String>,
    pub wasm_class: String,
    pub wasm_status: String,
    pub scan_ok: bool,
    pub has_flow: bool,
    pub num_flows: usize,
    pub num_sinks: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_category: Option<String>,
    #[serde(default)]
    pub stars: u64,
    #[serde(default)]
    pub dep_count: u32,
    #[serde(default)]
    pub ecosystem: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_ms: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_ms: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_ms: Option<u128>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusScanReport {
    pub run_id: String,
    pub total_repos: usize,
    pub resolved_repos: usize,
    pub scanned_repos: usize,
    pub scan_success_rate: f64,
    pub suspicious_rate: f64,
    pub by_wasm_class: std::collections::HashMap<String, ClassStats>,
    #[serde(default)]
    pub by_ecosystem: std::collections::HashMap<String, ClassStats>,
    #[serde(default)]
    pub by_failure_category: std::collections::HashMap<String, usize>,
    #[serde(default)]
    pub latency: LatencyStats,
    pub cases: Vec<CorpusScanCase>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LatencyStats {
    pub count: usize,
    pub build_ms_p50: u128,
    pub build_ms_p95: u128,
    pub scan_ms_p50: u128,
    pub scan_ms_p95: u128,
    pub total_ms_p50: u128,
    pub total_ms_p95: u128,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassStats {
    pub total: usize,
    pub scanned: usize,
    pub suspicious: usize,
}