pub mod bench;
pub mod metrics;
pub mod score;
pub mod suite;

pub use bench::{BenchReport, run_bench, run_id, write_bench_report};
pub use metrics::{Confusion, Label, Verdict};
pub use score::{ScenarioKind, label_for_scenario, scenario_from_name, score_case};
pub use suite::{SuiteId, resolve_suite};
