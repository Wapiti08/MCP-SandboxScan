use serde::{Deserialize, Serialize};
use crate::traint::source::TaintSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintedString {
    pub value: String,
    pub sources: Vec<TraintSource>,
}

