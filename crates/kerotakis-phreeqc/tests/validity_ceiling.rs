//! The aqueous model's validity boundaries, tested from both sides.
//!
//! Two lines the PHREEQC route used to cross silently: llnl.dat's
//! temperature parameterisation ends at 300 °C (curiosity th-022), and
//! a liquid that is mostly organic is not an aqueous solution with
//! impurities (th-057). On the far side of each line the engine now
//! stands aside and the honesty pass says why; on the near side the
//! chemistry still solves exactly as before.

#![cfg(feature = "engine")]

use kerotakis_core::script::parse_op;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(PhreeqcEquilibrator::new().expect("engine")),
        Box::new(HonestyEquilibrator),
    ])
}

fn run(script: &[&str]) -> (Bench, Vec<Event>) {
    let mut bench = Bench::new();
    let mut stack = stack();
    let mut all = Vec::new();
    for line in script {
        let op = parse_op(line).expect("parses").expect("an operator");
        let events = bench
            .step_with(op, &mut stack, &PermissiveScreen)
            .expect("steps");
        all.extend(events);
    }
    (bench, all)
}

fn apologies(events: &[Event]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::NotYetModeled { what, .. } => Some(what.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn superheated_sealed_water_takes_the_honesty_pass_not_an_extrapolation() {
    // th-022's shape: sealed water driven far past the databases' 300 °C
    // ceiling. The old behaviour was a PHREEQC convergence failure on
    // chemistry nobody parameterised; the new behaviour is a spoken
    // boundary — and the burst physics still gets to happen.
    let (_bench, events) = run(&[
        "add v1 water 100mL",
        "seal v1 101mL",
        "heat v1 500kJ",
        "look v1",
    ]);
    assert!(
        apologies(&events)
            .iter()
            .any(|w| w.contains("temperature ceiling")),
        "the ceiling is named: {:?}",
        apologies(&events)
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })),
        "standing aside is not failing: {events:?}"
    );
}

#[test]
fn water_at_150_c_still_solves() {
    // The near side of the same line: sealed water heated within the
    // parameterisation keeps its characterised solution.
    let (bench, events) = run(&["add v1 water 100mL", "seal v1 101mL", "heat v1 60kJ"]);
    let v = bench.vessel(VesselId(0)).unwrap();
    assert!(
        v.temperature.0 > 373.15 && v.temperature.0 < solve::AQUEOUS_MODEL_CEILING_K,
        "between boiling and the ceiling: {} K",
        v.temperature.0
    );
    assert!(v.solution.is_some(), "still characterised");
    assert!(
        !apologies(&events)
            .iter()
            .any(|w| w.contains("temperature ceiling")),
        "no apology on the near side: {:?}",
        apologies(&events)
    );
}

#[test]
fn salt_in_spirits_is_refused_as_mostly_organic() {
    // An ionic solid in a liquid that is almost entirely ethanol, with
    // no curated route to answer for it (permanganate has one, and an
    // apology after an answer is noise — core's permanganate tests pin
    // that side). The aqueous engine's activity models assume water as
    // the solvent; here they do not apply, and the refusal says so
    // instead of solving wine chemistry in a spirit still.
    let (bench, events) = run(&[
        "add v1 ethanol 0.02mol",
        "add v1 water 0.005mol",
        "add v1 NaCl 0.001mol",
        "look v1",
    ]);
    assert!(
        apologies(&events)
            .iter()
            .any(|w| w.contains("mostly organic")),
        "the dielectric boundary is named: {:?}",
        apologies(&events)
    );
    assert!(
        bench.vessel(VesselId(0)).unwrap().solution.is_none(),
        "no aqueous speciation is claimed for the mixture"
    );
}

#[test]
fn wine_strength_ethanol_still_solves() {
    // The near side of the dielectric line: water in the clear majority
    // keeps the aqueous engine on the case.
    let (bench, events) = run(&[
        "add v1 water 5mol",
        "add v1 ethanol 0.8mol",
        "add v1 NaCl 0.01mol",
        "look v1",
    ]);
    assert!(
        bench.vessel(VesselId(0)).unwrap().solution.is_some(),
        "wine-strength mixtures stay characterised"
    );
    assert!(
        !apologies(&events)
            .iter()
            .any(|w| w.contains("mostly organic")),
        "no apology on the near side: {:?}",
        apologies(&events)
    );
}

#[test]
fn a_solute_free_distillate_earns_no_speciation_apology() {
    // aq-078's shape: distilling ethanol-water concentrates the organic
    // in the receiver. There is nothing dissolved there to speciate, so
    // an apology about ionic speciation would be noise dressed as
    // honesty — the engine stands aside silently.
    let (_, events) = run(&[
        "new",
        "add v1 water 5mol",
        "add v1 ethanol 2mol",
        "distil v1 v2 0.3",
        "look v2",
    ]);
    assert!(
        !apologies(&events)
            .iter()
            .any(|w| w.contains("mostly organic")),
        "pure solvents draw no speciation apology: {:?}",
        apologies(&events)
    );
}
