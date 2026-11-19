//! Sodium sulfide on the bench: reduced sulfur stays reduced beside room
//! air, books as `HS-`, and acidification vents only the hydrogen sulfide
//! above saturation in the zero-time add step.
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
    let sulfide = moles(&bench, "HS-");
    assert!(
        (sulfide - 0.005).abs() < 1e-6,
        "sulfide must be kept as HS-: {sulfide}\n{steps:#?}"
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
    let kept = moles(&bench, "HS-");
    assert!(
        (kept + vented - 0.05).abs() < 1e-8,
        "sulfur is conserved between solution and the vented gas: {kept} + {vented}"
    );
    assert!(moles(&bench, "SO4-2") < 1e-12);
    assert!(noted(&steps[2], "not-modeled.sulfide-open-air"));
}
