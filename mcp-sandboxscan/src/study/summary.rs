use serde::{Deserialize, Serialize};

use super::matrix::StudyCaseResult;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StudySummary {
    pub total_cases: usize,
    pub scanned_cases: usize,
    pub failed_cases: usize,
    pub detected_cases: usize,
    pub clean_cases: usize,
    pub total_flows: usize,
}

impl StudySummary {
    pub fn from_cases(cases: &[StudyCaseResult]) -> Self {
        let total_cases = cases.len();
        let scanned_cases = cases.iter().filter(|case| case.error.is_none()).count();
        let failed_cases = total_cases.saturating_sub(scanned_cases);
        let detected_cases = cases
            .iter()
            .filter(|case| case.error.is_none() && case.has_external_to_prompt_flow)
            .count();
        let clean_cases = cases
            .iter()
            .filter(|case| case.error.is_none() && !case.has_external_to_prompt_flow)
            .count();
        let total_flows = cases.iter().map(|case| case.num_flows).sum();

        Self {
            total_cases,
            scanned_cases,
            failed_cases,
            detected_cases,
            clean_cases,
            total_flows,
        }
    }
}
