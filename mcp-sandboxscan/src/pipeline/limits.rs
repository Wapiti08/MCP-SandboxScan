use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ScanLimits {
    pub build_timeout: Option<Duration>,
    pub mcp_timeout: Option<Duration>,
}

impl ScanLimits {
    pub fn none() -> Self {
        Self {
            build_timeout: None,
            mcp_timeout: None,
        }
    }

    pub fn corpus_defaults() -> Self {
        Self {
            build_timeout: Some(Duration::from_secs(300)),
            mcp_timeout: Some(Duration::from_secs(60)),
        }
    }
}

impl Default for ScanLimits {
    fn default() -> Self {
        Self::none()
    }
}
