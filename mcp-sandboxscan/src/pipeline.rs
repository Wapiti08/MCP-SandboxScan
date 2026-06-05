use std::collections::HashMap;
use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::adapter::rust_wasi::RustWasiAdapter;
use crate::adapter::{AdaptationStatus, Adapter, BuildArtifact};
use crate::scan::dynamic::run_dynamic_scan;
use crate::scan::report::ScanReport;
use crate::subject::{Language, SubjectManifest};

pub fn scan_subject(
    subject: &SubjectManifest,
    env: &HashMap<String, String>,
    data_dir: Option<&Path>,
    max_output_bytes: usize,
)-> Result<ScanReport>{
    // ？=> error operator
    let adapter = select_adapter(subject)?;

    let adaptation = adapter
        .adapt(subject)
        .with_context(|| format!("failed to adapt subject {}", subject.name))?;


    if !matches!(adaptation.status, AdaptationStatus::DirectWasm | AdaptationStatus::WasmWithShim) {
        bail!(
            "subject {} is not available as wasm: status={:?}, blockers={:?}",
            adaptation.subject_name,
            adaptation.status,
            adaptation.blockers
        );
    }

    let Some(BuildArtifact::Wasm { wasm_path })= adaptation.artifact else {
        bail!(
            "subject {} did not produce a wasm artifact",
            adaptation.subject_name
        );
    };
    run_dynamic_scan(&wasm_path, data_dir, env, max_output_bytes)
        .with_context(|| format!("failed to scan wasm artifact {}", wasm_path.display()))
}

// return either successful result or error
fn select_adapter(subject: &SubjectManifest) -> Result<Box<dyn Adapter>> {
    match subject.language {
        Language::Rust => Ok(Box::new(RustWasiAdapter)),
        _ => bail!("no adapter implemented for language {:?}", subject.language),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_rust_env_leak_subject() {
        let raw = std::fs::read_to_string("case_studies/rust-env-leak/subject.toml")
            .expect("read subject manifest");
        let subject: SubjectManifest = toml::from_str(&raw).expect("parse subject manifest");

        let mut env = HashMap::new();
        env.insert(
            "DEMO_SECRET".to_string(),
            "SEKRET_0123456789abcdef".to_string(),
        );

        let report = scan_subject(&subject, &env, None, 4096).expect("scan subject");

        assert!(report.summary.has_external_to_prompt_flow);
        assert!(report.summary.num_flows > 0);
        assert!(report
            .flows
            .iter()
            .any(|flow| flow.source_id == "EnvVar: DEMO_SECRET"));
    }
}