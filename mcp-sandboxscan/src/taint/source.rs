use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TaintSource {
    FileRead {
        path: String,
        content: String,
    },
    EnvVar {
        key: String,
        value: String,
    },
    HttpFetch {
        url: String,
        content: String,
    },
    NetworkConnect {
        host: String,
        port: u16,
        protocol: String,
        content: String,
    },
    ToolInput {
        tool: String,
        path: String,
        content: String,
    },
}

impl TaintSource {
    pub fn content(&self) -> &str {
        match self {
            TaintSource::FileRead { content, .. } => content,
            TaintSource::EnvVar { value, .. } => value,
            TaintSource::HttpFetch { content, .. } => content,
            TaintSource::NetworkConnect { content, .. } => content,
            TaintSource::ToolInput { content, .. } => content,
        }
    }

    pub fn short_id(&self) -> String {
        match self {
            TaintSource::FileRead { path, .. } => format!("FileRead: {}", path),
            TaintSource::EnvVar { key, .. } => format!("EnvVar: {}", key),
            TaintSource::HttpFetch { url, .. } => format!("HttpFetch: {}", url),
            TaintSource::NetworkConnect {
                host,
                port,
                protocol,
                ..
            } => {
                format!("NetworkConnect: {protocol}://{host}:{port}")
            }
            TaintSource::ToolInput { tool, path, .. } => {
                format!("ToolInput: {tool} {path}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_source_content() {
        let src = TaintSource::EnvVar {
            key: "API_KEY".to_string(),
            value: "SECRET".to_string(),
        };

        assert_eq!(src.content(), "SECRET");
        assert!(src.short_id().starts_with("EnvVar: "));
    }

    #[test]
    fn network_connect_source_content() {
        let src = TaintSource::NetworkConnect {
            host: "evil.example".to_string(),
            port: 443,
            protocol: "https".to_string(),
            content: "https://evil.example/c2".to_string(),
        };
        assert_eq!(src.content(), "https://evil.example/c2");
        assert!(src.short_id().contains("evil.example"));
    }
}
