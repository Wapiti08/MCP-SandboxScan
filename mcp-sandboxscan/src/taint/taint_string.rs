use serde::{Deserialize, Serialize};
use crate::taint::source::TaintSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintedString {
    pub value: String,
    pub sources: Vec<TaintSource>,
}

