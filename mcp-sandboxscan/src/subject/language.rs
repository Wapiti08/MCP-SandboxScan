use serde::{Deserialize, Serialize};

// automatically do the ser/de process
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
// name pattern
#[serde(rename_all = "kebab-case")]
pub enum Language {
    Rust,
    Go,
    Python,
    TypeScript,
    JavaScript,
    Unknown,
}
