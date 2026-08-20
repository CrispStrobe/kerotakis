//! PLAN.md quotes numbers that this workspace computes, and prose copies of
//! computed values rot silently — README's did, PLAN's phase-coverage
//! figures went stale within a day of being written. The load-bearing ones
//! are pinned here against what computes them, on the same principle as
//! kerotakis-codex's README test: when this fails, update PLAN.md, not
//! this test.

/// Whitespace-normalised, so a re-wrapped paragraph is not a false alarm.
fn flatten(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn plan_quotes_the_real_phase_coverage() {
    let coverage = kerotakis_phreeqc::derived::phase_coverage();
    let plan = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../PLAN.md"),
    )
    .expect("PLAN.md at the repository root");
    let plan = flatten(&plan);

    // The provenance section's comparability sentence, and the drift
    // chronicle's "not a corner" figure.
    for sentence in [
        format!(
            "only {} of {} mineral phases exist in all three",
            coverage.shared, coverage.total
        ),
        format!(
            "{} of {} mineral phases exist in only some",
            coverage.total - coverage.shared,
            coverage.total
        ),
    ] {
        assert!(
            plan.contains(&sentence),
            "PLAN.md no longer matches derived::phase_coverage() — update the prose, \
             not this test. Expected it to contain: {sentence:?}"
        );
    }
}
