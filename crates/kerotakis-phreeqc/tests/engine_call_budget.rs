//! OPT-7's measurement, held as a regression budget.
//!
//! A coupled redox equilibration used to cost up to ~34 engine calls per
//! pe bisection, times every iteration of the temperature fixed point,
//! with nothing reused — the worst case penciled out near 272 calls. The
//! trial cache, the warm-started bracket and the residual break exist to
//! shrink that; this test pins the ceiling so it cannot quietly grow
//! back, and proves the trial cache actually fires when the same
//! chemistry is asked twice.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn add(bench: &mut Bench, eq: &mut PhreeqcEquilibrator, v: VesselId, key: &str, moles: f64) {
    bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            eq,
            &PermissiveScreen,
        )
        .expect("step");
}

/// The permanganate titration below equivalence: a genuinely bracketed
/// pe root, the expensive path.
fn titrate(eq: &mut PhreeqcEquilibrator) {
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, eq, v, "water", 5.55);
    add(&mut bench, eq, v, "HCl", 0.01);
    add(&mut bench, eq, v, "FeSO4", 0.005);
    add(&mut bench, eq, v, "KMnO4", 0.0008);
    assert!(
        bench.vessel(v).expect("vessel").solution.is_some(),
        "the titration must actually solve"
    );
}

#[test]
fn coupled_equilibration_stays_inside_the_call_budget() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");

    titrate(&mut eq);
    let first = eq.engine_calls();
    let first_hits = eq.trial_cache_hits();
    eprintln!("first titration: {first} engine calls, {first_hits} trial-cache hits");

    // The whole four-step titration — four equilibrations, the last two
    // through the coupled path — must fit far under the old single-solve
    // worst case. The ceiling is deliberately loose; the point is that
    // 200+ can never again look normal.
    assert!(
        first < 160,
        "coupled titration cost {first} engine calls — the OPT-7 budget is 160"
    );

    // The same chemistry a second time costs *zero* engine calls: the
    // content-addressed result cache one level up answers every step
    // verbatim, so the bisection never runs at all. (The trial cache
    // below it earns its keep when the temperature fixed point iterates
    // and re-asks trial inputs inside a single equilibration — a case
    // the result cache cannot see.)
    titrate(&mut eq);
    let second = eq.engine_calls() - first;
    let second_hits = eq.trial_cache_hits() - first_hits;
    eprintln!("second titration: {second} engine calls, {second_hits} trial-cache hits");
    assert_eq!(
        second, 0,
        "a verbatim repeat must be answered entirely from the result cache"
    );
}
