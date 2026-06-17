use crate::sandbox::exec_result::WasmExecResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvidence {
    pub backend: ExecutionBackend,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionBackend {
    Wasm,
    NativeStdio,
}

impl From<WasmExecResult> for ExecutionEvidence {
    fn from(exec: WasmExecResult) -> Self {
        Self {
            backend: ExecutionBackend::Wasm,
            stdout: exec.stdout,
            stderr: exec.stderr,
            exit_code: Some(exec.exit_code),
            duration_ms: exec.duration_ms,
        }
    }
}
