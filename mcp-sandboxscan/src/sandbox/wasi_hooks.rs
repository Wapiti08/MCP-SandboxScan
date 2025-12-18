use std::collections::HashMap;
use std::fs
use std::path::{Path, PathBuf};

use anyhow::Result;
use walkdir::WalkDir;

use crate::taint::source::TaintSource;

pub fn collect_env_sources(env: &HashMap<String, String>) -> Vec<TaintSource> {
    env.iter()
        .map(|(k, v)| TaintSource::EnvVar {
            // need own data, taint graph has longer lifetime than env
            key: k.clone(),
            value: v.clone(),
        })
        // put taintsources into a vec
        .collect()
}

/// take file contents under /data as source may be read by outside WASI
pub fn collect_file_sources(data_dir: Option<&Path>, max_bytes_per_file: usize) -> Result<Vec<TaintSource>> {
    // sources type can be inferred later
    let mut sources = vec![];

    let Some(root) = data_dir else {
        return Ok(sources);
    };

    for entry in WalkDir::new(root).into_iter().filter_map(|entry_result| entry_result.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }

        let path: PathBuf = entry.path().to_path_buf();
        let bytes = fs::read(&path).unwrap_or_default();
        
        if bytes.is_empty() {
            continue;
        }
        // truncate if larger than max bytes
        let truncated = if bytes.len() > max_bytes_per_file {
            // take only up to max bytes
            &bytes[..max_bytes_per_file]
        } else {
            &bytes[..]
        };

        let content = String::from_utf8_lossy(truncated).to_string();

        sources.push(TaintSource::FileRead {
            path: path.display().to_string(),
            content,
        });
    }

    Ok(sources)
}

/// extract "HTTP fetch intent" from output
/// record if any "fetch" or "http" keywords are found in stdout/stderr
pub fn collect_http_intents(stdout: &str, stderr: &str) -> Vec<TaintSource> {
    fn scan(s: &str) -> Vec<TaintSource> {
        // create out vec to save results
        let mut out=vec![];
        for line in s.lines() {
            let trimmed = line.trim();
            for prefix in ["HTTP_FETCH:", "FETCH:", "HTTP:"] {
                if let Some(rest) = trimmed.strip_prefix(prefix) {
                    let url = rest.trim().to_string();
                    if !url.is_empty() {
                        out.push(TaintSource::HttpFetchIntent { 
                            url,
                            content: "<intent-only>".to_string(),
                            });
                    }
                }
            }
        }
        out
    }
    let mut res = scan(stdout);
    res.extend(scan(stderr));
    res
}