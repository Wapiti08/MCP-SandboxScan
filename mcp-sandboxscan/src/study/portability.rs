use serde::{Deserialize, Serialize};

use crate::adapter::AdaptationStatus;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WasmPortabilityStatus {
    DirectWasm,
    WasmWithShim,
    NativeOnly,
    Unsupported,
    Failed,
}

impl From<AdaptationStatus> for WasmPortabilityStatus {
    fn from(status: AdaptationStatus) -> Self {
        match status {
            AdaptationStatus::DirectWasm => Self::DirectWasm,
            AdaptationStatus::WasmWithShim => Self::WasmWithShim,
            AdaptationStatus::NativeOnly => Self::NativeOnly,
            AdaptationStatus::Unsupported => Self::Unsupported,
            AdaptationStatus::Failed => Self::Failed,
        }
    }
}
