use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag="type")]
pub enum TaintSource {
    FileRead {path: String, content: String},
    EnvVar {key: String, value: String},
    HttpFetch {url: String, content: String},
}

impl TaintSource {
    pub fn content(&self) -> &str {
        match self {
            // care content only
            TaintSource::FileRead {content, ..} => content,
            TaintSource::EnvVar {value, ..} => value,
            TaintSource::HttpFetch {content, ..} => content,
        }
    }

    pub fn short_id(&self) -> String {
        match self {
            TaintSource::FileRead {path, ..} => format!("FileRead: {}", path),
            TaintSource::EnvVar {key, ..} => format!("EnvVar: {}", key),
            TaintSource::HttpFetch {url, ..} => format!("HttpFetch: {}", url),
        }
    }
}