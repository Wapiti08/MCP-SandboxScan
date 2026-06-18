use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::scan::compare::discover_case_studies;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuiteId {
    Full,
    WasiCore,
    SmallTs,
}

impl SuiteId {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "full" => Ok(Self::Full),
            "wasi-core" => Ok(Self::WasiCore),
            "small-ts" => Ok(Self::SmallTs),
            other => bail!("unknown suite: {other} (supported: full, wasi-core, small-ts)"),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::WasiCore => "wasi-core",
            Self::SmallTs => "small-ts",
        }
    }
}

pub fn resolve_suite(manifest_dir: &Path, suite: SuiteId) -> Result<Vec<PathBuf>> {
    let paths: Vec<PathBuf> = match suite {
        SuiteId::Full => discover_case_studies(manifest_dir)?,
        SuiteId::WasiCore => vec![
            PathBuf::from("case_studies/rust-benign/subject.toml"),
            PathBuf::from("case_studies/rust-env-leak/subject.toml"),
            PathBuf::from("case_studies/rust-file-exfil/subject.toml"),
            PathBuf::from("case_studies/python-benign/subject.toml"),
            PathBuf::from("case_studies/python-env-leak/subject.toml"),
            PathBuf::from("case_studies/python-file-exfil/subject.toml"),
        ],
        SuiteId::SmallTs => vec![
            PathBuf::from("case_studies/ts-benign/subject.toml"),
            PathBuf::from("case_studies/ts-env-leak/subject.toml"),
            PathBuf::from("case_studies/ts-file-exfil/subject.toml"),
            PathBuf::from("case_studies/ts-c2-beacon/subject.toml"),
        ],
    };

    let paths: Vec<PathBuf> = paths
        .into_iter()
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                manifest_dir.join(path)
            }
        })
        .collect();

    for path in &paths {
        if !path.exists() {
            bail!("suite subject not found: {}", path.display());
        }
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_suite_discovers_all_case_studies() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let paths = resolve_suite(&manifest_dir, SuiteId::Full).expect("resolve full suite");
        assert_eq!(paths.len(), 30);
    }
}
