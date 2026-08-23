//! CAP-2: Data parallelism via rayon.
//!
//! Multi-vessel benchmarks and batch lesson replay. Each lesson runs
//! on its own independent bench — no shared mutable state.

use rayon::prelude::*;

use crate::bench::Bench;
use crate::ops::Event;
use crate::script;

/// Run multiple .lab scripts in parallel and return their event streams.
///
/// Each script gets its own bench — no shared state, no data races.
/// The results are returned in the same order as the input scripts.
pub fn run_lessons_parallel(scripts: &[&str]) -> Vec<Result<Vec<Vec<Event>>, String>> {
    scripts
        .par_iter()
        .map(|script| {
            let mut bench = Bench::default();
            let mut all_events = Vec::new();
            for line in script.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                match script::parse_op(line) {
                    Ok(Some(op)) => match bench.step(op) {
                        Ok(events) => all_events.push(events),
                        Err(e) => return Err(format!("step failed: {e}")),
                    },
                    Ok(None) => {}
                    Err(e) => return Err(format!("parse failed: {e}")),
                }
            }
            Ok(all_events)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_lessons_produce_independent_results() {
        let script1 = "add v1 water 100mL\nmeasure v1 thermometer";
        let script2 = "add v1 water 200mL\nmeasure v1 balance";
        let results = run_lessons_parallel(&[script1, script2]);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }
}
