//! Sodium sulfide on the bench: reduced sulfur stays reduced beside room
//! air, books as `HS-`, and acidification vents only the hydrogen sulfide
//! above saturation in the zero-time add step, and (since 2026-11-20) the
//! bounded acid/sulfide endpoint vents newly formed H2S(aq) below it in a
//! dilute open beaker.
//!
//! Written 2026-11-19 without running the engine (the authoring lane runs
//! no compiler). The expected numbers are estimates by reading, stated
//! with their arithmetic, and the tolerances are deliberately wide enough
//! to survive the activity model while still failing on the wrong chemistry
//! (sulfide oxidised to sulfate, sulfide lost, or every mole vented).
//!
//! - 0.005 mol Na2S in 100 mL: pH about 12.5-13 (S2-/HS- hydrolysis), all
//!   S(-2) retained as `HS-` through the add and a 600 s wait, no sulfate,
//!   no hydrogen sulfide event, and the `not-modeled.sulfide-open-air` note.
//! - 0.05 mol Na2S, then 0.2 mol HCl, in 100 mL: 0.05 mol H2S formed,
//!   0.5 mol/kgw against a saturation of about 0.10 mol/kgw at 1 atm
//!   (wateq4f `H2S(g)` log_k -0.997), so the add step vents about
//!   (0.5 - 0.10) x 0.1 kg = 0.040 mol. Salting and activity effects at
//!   I about 1.5 move this by a few percent; the window is 0.030-0.046.
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
        .moles_of(&SpeciesId::new(key))
        .0
}

/// Reduced sulfur in the vessel, whichever member of the S(-2) split books
/// it: `H2S(aq)`, `HS-` or `S-2` (since 2026-11-20).
fn reduced(bench: &Bench) -> f64 {
    ["H2S(aq)", "HS-", "S-2"]
        .iter()
        .map(|key| moles(bench, key))
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

#[test]
fn an_open_beaker_of_sodium_sulfide_keeps_its_sulfide() {
    // 0.005 mol x 78.0395 g/mol = 0.390 g.
    let (bench, steps) = run(&["add v1 water 100mL", "add v1 Na2S 0.3902g", "wait 600s"]);
    let sulfide = reduced(&bench);
    assert!(
        (sulfide - 0.005).abs() < 1e-6,
        "sulfide must be kept as reduced sulfur: {sulfide}\n{steps:#?}"
    );
    // At pH 12-13 the split is HS- and S-2; the neutral share is a trace.
    // How much is S-2 depends on the route's HS-/S-2 constant (wateq4f
    // -12.918, minteq.v4 -17.3), so only its sign is pinned here.
    assert!(
        moles(&bench, "H2S(aq)") < 0.01 * sulfide,
        "an alkaline sulfide solution holds almost no H2S(aq)"
    );
    let ion = moles(&bench, "HS-");
    let bare = moles(&bench, "S-2");
    assert!(
        ion + bare > 0.99 * sulfide && ion > 0.0,
        "alkaline sulfide is HS- and S-2: HS- {ion}, S-2 {bare} of {sulfide}"
    );
    assert!(moles(&bench, "Na2S") < 1e-12, "the salt dissolves");
    assert!(
        moles(&bench, "SO4-2") < 1e-12,
        "no air oxidation to sulfate"
    );
    let ph = bench
        .vessel(VesselId(0))
        .unwrap()
        .solution
        .clone()
        .expect("a sulfide solution is characterised")
        .ph;
    assert!(
        (11.5..13.5).contains(&ph),
        "alkaline sulfide solution: pH {ph}"
    );
    for events in &steps[1..] {
        assert!(evolved(events, "hydrogen_sulfide") < 1e-9, "{events:#?}");
        assert!(noted(events, "not-modeled.sulfide-open-air"), "{events:#?}");
        assert!(
            !noted(events, "not-modeled.sulfide-fast-endpoint"),
            "alkaline sulfide does not fizz: {events:#?}"
        );
    }
}

#[test]
fn acidifying_concentrated_sulfide_vents_only_the_supersaturated_hydrogen_sulfide() {
    // 0.05 mol x 78.0395 g/mol = 3.902 g.
    let (bench, steps) = run(&[
        "add v1 water 100mL",
        "add v1 Na2S 3.902g",
        "add v1 HCl 0.2mol",
    ]);
    let vented = evolved(&steps[2], "hydrogen_sulfide");
    assert!(
        (0.030..0.046).contains(&vented),
        "add-step H2S vent {vented} mol, expected about 0.040\n{:#?}",
        steps[2]
    );
    // The balance is against the sulfide actually added: 3.902 g at the
    // registry molar mass is 0.0500003 mol, not the rounded 0.05.
    let added: f64 = steps[1]
        .iter()
        .filter_map(|event| match event {
            Event::Added { species, moles, .. } if species.0 == "Na2S" => Some(moles.0),
            _ => None,
        })
        .sum();
    assert!((added - 0.05).abs() < 1e-5, "added {added} mol Na2S");
    let kept = reduced(&bench);
    // 0.2 mol HCl on 0.05 mol sulfide leaves strong acid in excess: what
    // stays dissolved is the neutral H2S(aq), not HS-.
    assert!(
        moles(&bench, "H2S(aq)") > 0.95 * kept,
        "acidified sulfide books as H2S(aq): {} of {kept}",
        moles(&bench, "H2S(aq)")
    );
    assert!(moles(&bench, "S-2") < 1e-12 * kept.max(1.0));
    assert!(
        (kept + vented - added).abs() < 1e-8,
        "sulfur is conserved between solution and the vented gas: {kept} + {vented} vs {added}"
    );
    assert!(moles(&bench, "SO4-2") < 1e-12);
    assert!(noted(&steps[2], "not-modeled.sulfide-open-air"));
}

#[test]
fn reduced_sulfur_survives_a_second_step_in_every_split_member() {
    // The round trip the split used to block: a neutral share booked on
    // step one must re-enter the solve on step two. Half-neutralised
    // sulfide (pH near pKa1, about 7) holds real H2S(aq) and real HS-.
    // Sealed since 2026-11-20: in an open beaker the newly formed H2S(aq)
    // now leaves through the acid/sulfide endpoint, and this test is about
    // the split's round trip, not about the gas boundary.
    let (bench, steps) = run(&[
        "add v1 water 100mL",
        "add v1 Na2S 0.3902g",
        "seal v1 500mL",
        "add v1 HCl 0.0075mol",
        "wait 60s",
        "wait 60s",
    ]);
    let total = reduced(&bench);
    assert!(
        (total - 0.005).abs() < 1e-6,
        "S(-2) conserved across steps: {total}\n{steps:#?}"
    );
    let neutral = moles(&bench, "H2S(aq)");
    let ion = moles(&bench, "HS-");
    assert!(
        neutral > 0.05 * total && ion > 0.05 * total,
        "{neutral} / {ion}"
    );
    assert!(moles(&bench, "SO4-2") < 1e-12);
}

fn failed(events: &[Event]) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::SolverFailed { .. }))
}

#[test]
fn acidified_dilute_sulfide_in_an_open_beaker_vents_its_hydrogen_sulfide() {
    // 0.005 mol Na2S and 0.02 mol HCl in 100 mL: strong acid in excess, so
    // essentially all 0.005 mol S(-2) becomes H2S(aq), 0.05 mol/kgw against
    // a saturation near 0.1 mol/kgw. No bubbles form, so before the endpoint
    // nothing left; the bounded endpoint now vents the newly formed neutral
    // share (the smell of an acidified sulfide), capped by acid and sulfide.
    let (bench, steps) = run(&[
        "add v1 water 100mL",
        "add v1 Na2S 0.3902g",
        "add v1 HCl 0.02mol",
    ]);
    let added: f64 = steps[1]
        .iter()
        .filter_map(|event| match event {
            Event::Added { species, moles, .. } if species.0 == "Na2S" => Some(moles.0),
            _ => None,
        })
        .sum();
    assert!(!failed(&steps[2]), "{:#?}", steps[2]);
    let vented = evolved(&steps[2], "hydrogen_sulfide");
    assert!(
        vented > 0.9 * added && vented <= added + 1e-9,
        "acidified sulfide vents its H2S: {vented} of {added}\n{:#?}",
        steps[2]
    );
    assert!(
        noted(&steps[2], "not-modeled.sulfide-fast-endpoint"),
        "{:#?}",
        steps[2]
    );
    let kept = reduced(&bench);
    assert!(kept < 0.1 * added, "S(-2) left the solution: kept {kept}");
    assert!(
        (kept + vented - added).abs() < 1e-8,
        "sulfur is conserved between solution and the vented gas: {kept} + {vented} vs {added}"
    );
    assert!(moles(&bench, "SO4-2") < 1e-12);
    // The carbon endpoint is not involved.
    assert!(evolved(&steps[2], "CO2") < 1e-9);
    assert!(!noted(&steps[2], "not-modeled.carbonate-fast-endpoint"));
}

#[test]
fn acidified_sulfide_in_a_sealed_vessel_keeps_its_reduced_sulfur() {
    let (bench, steps) = run(&[
        "add v1 water 100mL",
        "add v1 Na2S 0.3902g",
        "seal v1 500mL",
        "add v1 HCl 0.02mol",
    ]);
    assert!(!failed(&steps[3]), "{:#?}", steps[3]);
    assert!(
        evolved(&steps[3], "hydrogen_sulfide") < 1e-9,
        "{:#?}",
        steps[3]
    );
    assert!(!noted(&steps[3], "not-modeled.sulfide-fast-endpoint"));
    // Whatever share a finite headspace holds stays in the vessel.
    let total = reduced(&bench) + moles(&bench, "hydrogen_sulfide");
    assert!(
        (total - 0.005).abs() < 1e-5,
        "a sealed vessel keeps its sulfide: {total}\n{steps:#?}"
    );
}
