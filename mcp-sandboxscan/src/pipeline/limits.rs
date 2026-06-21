use std::time::Duration;

use crate::mcp::explore::ExplorationConfig;

#[derive(Debug, Clone)]
pub struct ScanLimits {
    pub build_timeout: Option<Duration>,
    pub mcp_timeout: Option<Duration>,
    pub exploration: ExplorationConfig,
}

impl ScanLimits {
    pub fn none() -> Self {
        Self {
            build_timeout: None,
            mcp_timeout: None,
            exploration: ExplorationConfig::disabled(),
        }
    }

    pub fn corpus_defaults() -> Self {
        Self {
            build_timeout: Some(Duration::from_secs(300)),
            mcp_timeout: Some(Duration::from_secs(60)),
            exploration: ExplorationConfig::disabled(),
        }
    }
}

impl Default for ScanLimits {
    fn default() -> Self {
        Self::none()
    }
}
