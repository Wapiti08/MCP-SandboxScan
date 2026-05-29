use std::path::PathBuf;

use serde::{Deserialize, Serialize};
// load from parent folder
use super::capability::Capability;
use super::language::Language;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectManifest {
    pub name: String,
    pub language: Language,
    pub source_dir: PathBuf,
    pub entrypoint: Option<String>,
    pub build: Option<BuildSpec>,
    pub run: Option<RunSpec>,
    #[serde(default)]
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSpec {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSpec {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

// test code
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_rust_env_leak_manifest() {
        let raw = std::fs::read_to_string("case_studies/rust-env-leak/subject.toml")
            .expect("read subject.toml");
        let manifest: SubjectManifest = toml::from_str(&raw)
            .expect("parse subject.toml");
        assert_eq!(manifest.name, "rust-env-leak");
        assert_eq!(manifest.language, Language::Rust);
        assert!(manifest.capabilities.contains(&Capability::Env));
        assert!(manifest.capabilities.contains(&Capability::Stdio));
        assert!(manifest.build.is_some());
    }
}