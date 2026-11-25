//! G6: ammonia from an ammonium salt and a strong base leaves an OPEN
//! beaker across the gas boundary, as the film clock's outward `NH3`
//! parcel, and only an open one.
//!
//! 0.02 mol NH4Cl + 0.02 mol NaOH in 100 mL: pH about 11.2, almost all of
//! the nitrogen as neutral NH3(aq). With room air ammonia-free, the still
//! surface carries `n0 (1 - exp(-t/tau))`, `tau = V/(k_L A)`; for this
//! beaker that is about 1.3 % of the NH3(aq) per minute. The bounds below
//! are deliberately wide (the rate is a stated liquid-film upper bound);
//! they fail on the wrong chemistry: nothing leaving, everything leaving,
//! nitrogen lost, or ammonia escaping a sealed vessel or a neutral salt.
#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
}

fn run(script: &[&str]) -> (Bench, Vec<Vec<Event>>) {
    let mut bench = Bench::new();
    let mut stack = stack();
    let mut steps = Vec::new();
    for op in script {
        steps.push(
            bench
                .step_with(
                    kerotakis_core::script::parse_op(op)
                        .expect("parse")
                        .expect("a known verb"),
                    &mut stack,
                    &PermissiveScreen,
                )
                .expect("step"),
        );
    }
    (bench, steps)
}

fn moles(bench: &Bench, key: &str) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .contents
        .iter()
        .filter(|p| p.species.0 == key)
        .map(|p| p.moles.0)
        .sum()
}

fn evolved(events: &[Event], key: &str) -> f64 {
    events
        .iter()
        .filter_map(|event| match event {
            Event::GasEvolved { species, moles, .. } if species.0 == key => Some(moles.0),
            _ => None,
        })
        .sum()
}

fn noted(events: &[Event], key: &str) -> bool {
    events.iter().any(|event| {
        matches!(event, Event::NotYetModeled { reason: Some(reason), .. } if reason.key == key)
    })
}

fn failed(events: &[Event]) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::SolverFailed { .. }))
}

const NOTE: &str = "not-modeled.ammonia-open-film";

#[test]
fn ammonium_chloride_and_sodium_hydroxide_vent_ammonia_from_an_open_beaker() {
    let (bench, steps) = run(&[
        "add v1 water 100mL",
        "add v1 NH4Cl 0.02mol",
        "add v1 NaOH 0.02mol",
        "wait 60s",
        "heat v1 20kJ",
        "wait 60s",
    ]);
    for (i, step) in steps.iter().enumerate() {
        assert!(!failed(step), "step {i}: {step:#?}");
    }
    let cold = evolved(&steps[3], "NH3");
    let warm = evolved(&steps[5], "NH3");
    for (label, vented, step) in [("cold", cold, &steps[3]), ("warm", warm, &steps[5])] {
        assert!(
            vented > 5e-5 && vented < 2e-3,
            "{label} wait vents a bounded share of 0.02 mol: {vented}\n{step:#?}"
        );
        assert!(noted(step, NOTE), "{label}: typed rate boundary\n{step:#?}");
    }
    // Zero-time steps are not a gas boundary for this film parcel.
    assert_eq!(evolved(&steps[2], "NH3"), 0.0);
    // Nitrogen: what is left plus what left is what was added.
    let kept = moles(&bench, "NH3") + moles(&bench, "NH4+");
    assert!(
        (kept + cold + warm - 0.02).abs() < 1e-9,
        "nitrogen conserved: {kept} + {cold} + {warm}"
    );
    assert!(moles(&bench, "Na+") > 0.0199);
    assert!(moles(&bench, "Cl-") > 0.0199);
}

#[test]
fn a_sealed_vessel_keeps_its_ammonia() {
    let (bench, steps) = run(&[
        "add v1 water 100mL",
        "add v1 NH4Cl 0.02mol",
        "seal v1 500mL",
        "add v1 NaOH 0.02mol",
        "wait 60s",
    ]);
    assert!(!failed(&steps[4]), "{:#?}", steps[4]);
    assert_eq!(evolved(&steps[4], "NH3"), 0.0, "{:#?}", steps[4]);
    assert!(!noted(&steps[4], NOTE));
    let total = moles(&bench, "NH3") + moles(&bench, "NH4+");
    assert!(
        (total - 0.02).abs() < 1e-9,
        "all ammonia nitrogen stays in the sealed vessel: {total}"
    );
}

#[test]
fn ammonium_chloride_alone_gives_off_no_ammonia() {
    let (_bench, steps) = run(&["add v1 water 100mL", "add v1 NH4Cl 0.02mol", "wait 600s"]);
    assert!(!failed(&steps[2]), "{:#?}", steps[2]);
    assert_eq!(evolved(&steps[2], "NH3"), 0.0, "{:#?}", steps[2]);
    assert!(!noted(&steps[2], NOTE));
}

fn solid(bench: &Bench, key: &str) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum()
}

/// Nitrate is nitrogen in another valence. Its N(V) is an independent
/// owner beside the N(-III) the vent spends, not a reason to refuse the
/// whole projection (after #901 every later solve of such a vessel failed
/// with "clock CO2 lacks a certified native component projection").
#[test]
fn ammonium_nitrate_and_sodium_hydroxide_vent_ammonia_and_keep_the_nitrate() {
    let (bench, steps) = run(&[
        "add v1 water 100mL",
        "add v1 NH4NO3 0.02mol",
        "add v1 NaOH 0.02mol",
        "wait 60s",
        "wait 60s",
    ]);
    for (i, step) in steps.iter().enumerate() {
        assert!(!failed(step), "step {i}: {step:#?}");
    }
    let vented = evolved(&steps[3], "NH3") + evolved(&steps[4], "NH3");
    assert!(vented > 5e-5 && vented < 2e-3, "bounded vent: {vented}");
    let kept = moles(&bench, "NH3") + moles(&bench, "NH4+");
    assert!(
        (kept + vented - 0.02).abs() < 1e-9,
        "ammonia nitrogen conserved: {kept} + {vented}"
    );
    assert!(
        (moles(&bench, "NO3-") - 0.02).abs() < 1e-9,
        "nitrate untouched: {}",
        moles(&bench, "NO3-")
    );
}

/// C3000 exp113: silver chloride dissolves in ammonia, the open beaker
/// vents some ammonia while stirred, and acetic acid then re-precipitates
/// the silver chloride. Silver nitrate puts N(V) beside the N(-III).
#[test]
fn silver_chloride_dissolved_in_ammonia_reprecipitates_with_acetic_acid() {
    let (bench, steps) = run(&[
        "add v1 water 5mL",
        "add v1 NaCl 0.0002mol",
        "add v1 AgNO3 0.0002mol",
        "add v1 NH3 0.005mol",
        "stir v1",
        "add v1 CH3COOH 0.01mol",
    ]);
    for (i, step) in steps.iter().enumerate() {
        assert!(!failed(step), "step {i}: {step:#?}");
    }
    assert!(
        solid(&bench, "AgCl") > 1.9e-4,
        "AgCl back out of solution: {}",
        solid(&bench, "AgCl")
    );
}
