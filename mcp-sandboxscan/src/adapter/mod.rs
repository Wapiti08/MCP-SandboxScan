pub mod go_wasi;
pub mod native_mcp;
pub mod python_wasi;
pub mod rust_wasi;
pub mod typescript_wasi;
pub mod unsupported;

use std::path::PathBuf;

use anyhow::Result;

use crate::subject::{Language, SubjectManifest};

// define common artifact for individual artifact
#[derive(Debug, Clone)]
pub enum BuildArtifact {
    Wasm {
        wasm_path: PathBuf,
    },
    PythonWasm {
        interpreter_wasm: PathBuf,
        work_dir: PathBuf,
        argv: Vec<String>,
    },
    NativeCommand {
        command: String,
        args: Vec<String>,
    },
    Unsupported {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdaptationStatus {
    DirectWasm,
    WasmWithShim,
    NativeOnly,
    Unsupported,
    Failed,
}

#[derive(Debug, Clone)]
pub struct AdaptationReport {
    pub subject_name: String,
    pub language: Language,
    pub status: AdaptationStatus,
    pub artifact: Option<BuildArtifact>,
    pub notes: Vec<String>,
    pub blockers: Vec<String>,
}


pub trait Adapter {
    fn name(&self) -> &'static str;
    fn adapt(&self, subject: &SubjectManifest) -> Result<AdaptationReport>;
}