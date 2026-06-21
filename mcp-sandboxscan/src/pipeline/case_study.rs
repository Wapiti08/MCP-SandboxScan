use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::subject::{Capability, SubjectManifest};

pub fn needs_data_dir(subject: &SubjectManifest) -> bool {
    subject
        .capabilities
        .iter()
        .any(|cap| matches!(cap, Capability::FileRead | Capability::FileWrite))
}

pub fn resolve_data_dir(
    manifest_dir: &Path,
    subject: &SubjectManifest,
    data_dir: Option<&Path>,
) -> Result<Option<PathBuf>> {
    if let Some(dir) = data_dir {
        return Ok(Some(dir.to_path_buf()));
    }
    if !needs_data_dir(subject) {
        return Ok(None);
    }

    let dir = manifest_dir.join("data");
    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let secret = dir.join("secret.txt");
    if !secret.exists() {
        std::fs::write(&secret, "top-secret-0123456789abcdef")
            .with_context(|| format!("failed to write {}", secret.display()))?;
    }
    Ok(Some(dir))
}

pub fn default_env_for_subject(
    subject: &SubjectManifest,
    env: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut out = env.clone();

    if subject.capabilities.contains(&Capability::FileRead) && !out.contains_key("FILE_TO_READ") {
        out.insert("FILE_TO_READ".to_string(), "secret.txt".to_string());
    }

    if !out.contains_key("DEMO_SECRET")
        && (subject.name.contains("env-leak")
            || subject.name.contains("env_leak")
            || subject.name.contains("c2-beacon")
            || subject.name.contains("c2_beacon"))
    {
        out.insert(
            "DEMO_SECRET".to_string(),
            "SEKRET_0123456789abcdef".to_string(),
        );
    }

    out
}
