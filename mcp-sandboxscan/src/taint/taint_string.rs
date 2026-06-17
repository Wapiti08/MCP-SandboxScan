use crate::taint::source::TaintSource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintedString {
    pub value: String,
    pub sources: Vec<TaintSource>,
}
