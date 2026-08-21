//! The release-one chemistry promise, exercised live and then replayed only
//! from the exact cache entries produced by that live run.

#![cfg(feature = "engine")]

use kerotakis_phreeqc::{acceptance::run_r1_acceptance, PhreeqcEquilibrator};

fn assert_r1(report: &kerotakis_phreeqc::acceptance::R1AcceptanceReport) {
    assert_eq!(report.schema, 1);
    assert_eq!(
        report
            .cases
            .iter()
            .map(|case| case.id.as_str())
            .collect::<Vec<_>>(),
        [
            "limewater",
            "carbonated_bottle",
            "surface_release",
            "softener_breakthrough",
            "partial_freezing",
        ]
    );
    assert!(
        report.passed(),
        "R1 acceptance failures:\n{}",
        serde_json::to_string_pretty(report).expect("serialise report")
    );
}

#[test]
fn r1_scenarios_run_live_and_replay_without_new_solver_results() {
    let mut live = PhreeqcEquilibrator::new().expect("live engine");
    let live_report = run_r1_acceptance(&mut live);
    assert_r1(&live_report);

    let cache = live.export_cache();
    assert!(!cache.entries.is_empty(), "R1 run must populate the cache");

    let mut replay = PhreeqcEquilibrator::new().expect("replay engine");
    assert_eq!(replay.import_cache(cache.clone()), cache.entries.len());
    let before = replay.cache_len();
    let replay_report = run_r1_acceptance(&mut replay);
    assert_r1(&replay_report);
    assert_eq!(replay_report, live_report, "cache replay must be exact");
    assert_eq!(
        replay.cache_len(),
        before,
        "cache replay computed a result that was not pre-warmed"
    );
    assert!(replay.cache_hits() > 0);
}
