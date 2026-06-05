use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WasmPortabilityStatus {
    DirectWasm,
    WasmWithShim,
    NativeOnly,
    Unsupported,
    Failed,
}
