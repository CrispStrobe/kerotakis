//! Provenance and multiple paths: every answer says where it came from,
//! and the same question can be put to every dataset that can express it
//! (PLAN.md: be open about sources, offer the paths rather than asserting
//! one).

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::{PathOutcome, PhreeqcEquilibrator};

const WATEQ_DATASET: &str = "wateq4f.dat plus USBM IC 9429 reference-temperature complexes, with the reviewed Sander HBr gas-uptake slice";
const PITZER_DATASET: &str = "pitzer.dat, with the reviewed Sander HBr gas-uptake slice";

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

#[test]
fn every_answer_carries_its_provenance() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 55.51);
    add(&mut bench, &mut eq, v, "NaCl", 0.05);

    let p = bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("characterised")
        .provenance
        .expect("provenance recorded");
    assert!(p.engine.contains("PHREEQC"));
    assert_eq!(p.dataset, WATEQ_DATASET);
    for source in ["usbm-ic9429-complexes-25c", "sander-2023-hbr-reference"] {
        assert!(
            p.dataset_sources.iter().any(|id| id == source),
            "missing {source}: {p:?}"
        );
    }
    assert!(p.model.contains("Debye"), "model named: {}", p.model);
    assert!(!p.routing.is_empty(), "routing reason given");
    assert!(
        !p.dataset_sources.is_empty(),
        "the dataset's own citations were captured"
    );
}

#[test]
fn concentrated_brine_is_routed_to_pitzer_and_says_so() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 55.51);
    add(&mut bench, &mut eq, v, "NaCl", 8.0);

    let p = bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .unwrap()
        .provenance
        .unwrap();
    assert_eq!(p.dataset, PITZER_DATASET);
    assert!(p
        .dataset_sources
        .iter()
        .any(|id| id == "sander-2023-hbr-reference"));
    assert!(
        !p.dataset_sources
            .iter()
            .any(|id| id == "usbm-ic9429-complexes-25c"),
        "the Pitzer route does not load the USBM complex extension"
    );
    assert!(p.model.contains("Pitzer"));
    assert!(
        p.routing.contains("concentrated"),
        "routing explains itself: {}",
        p.routing
    );
}

#[test]
fn the_paths_disagree_and_all_three_are_reported() {
    // The pedagogical point: three thermodynamic datasets give three
    // different answers for saturated brine, and each states the model it
    // applies. Showing the disagreement is more honest than picking one
    // silently.
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 55.51);
    add(&mut bench, &mut eq, v, "NaCl", 8.0);

    let vessel = bench.vessel(v).unwrap().clone();
    let paths = eq.compare_paths(&vessel);
    assert_eq!(paths.len(), 3, "every dataset is asked");

    let solved: Vec<(String, f64)> = paths
        .iter()
        .filter_map(|p| match &p.outcome {
            PathOutcome::Solved { phases, .. } => phases
                .iter()
                .find(|(n, _)| n == "Halite")
                .map(|(_, m)| (p.dataset.clone(), *m)),
            _ => None,
        })
        .collect();
    assert_eq!(solved.len(), 3, "all three can express NaCl in water");

    let values: Vec<f64> = solved.iter().map(|(_, m)| *m).collect();
    let spread = values.iter().cloned().fold(f64::MIN, f64::max)
        - values.iter().cloned().fold(f64::MAX, f64::min);
    assert!(
        spread > 1.0,
        "the datasets genuinely disagree in this regime: {solved:?}"
    );
}

#[test]
fn a_dataset_that_cannot_express_the_question_says_so() {
    // pitzer.dat has no silver: it must decline explicitly rather than be
    // skipped or, worse, answer wrongly.
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 55.51);
    add(&mut bench, &mut eq, v, "AgNO3", 0.01);

    let vessel = bench.vessel(v).unwrap().clone();
    let paths = eq.compare_paths(&vessel);
    let pitzer = paths
        .iter()
        .find(|p| p.dataset == PITZER_DATASET)
        .expect("pitzer reported");
    match &pitzer.outcome {
        PathOutcome::CannotExpress { missing_elements } => {
            assert!(
                missing_elements.iter().any(|e| e == "Ag"),
                "names what it lacks: {missing_elements:?}"
            );
        }
        other => panic!("expected CannotExpress, got {other:?}"),
    }
}

/// Which dataset answers this beaker, on the wire — and only when it
/// changes.
///
/// `Event::SolutionRouted` exists because the provenance a learner would
/// most want was the one the web could not see: `ProvenanceDrawer.svelte`
/// reads `event.provenance`, and the only event that carried one was the
/// combustion `ThermalEquilibrium`. The aqueous routing lived on
/// `vessel.solution.provenance` and reached `kero explain` and nothing a
/// reader reads in a browser.
///
/// The design is the emission rule, not the wire format, so BOTH halves
/// are tested here. A test that only checked it fires would pass just as
/// happily if it fired on every solve — which is what it must not do:
/// `finalize_solution_info` runs five times over the three commands of
/// `aq-023` alone.
fn routed(events: &[Event], v: VesselId) -> Vec<&vessel::Provenance> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::SolutionRouted { vessel, provenance } if *vessel == v => Some(provenance),
            _ => None,
        })
        .collect()
}

/// Adding water alone announces its routing, once.
///
/// **This is the reversal of #653's scope, ruled by the owner on
/// 2026-09-18.** That PR left this path silent on the grounds that an
/// empty event list is what keeps routine water setup from being filed by
/// `kero coverage` as a computed answer — but the consequence was a beaker
/// that held a `Provenance` in its state saying which relation answered it
/// and no reader who could ever reach it. The count is
/// `solve::answer_event_count` now, so a routing announcement is not a
/// result and coverage is unmoved; the reader is told.
///
/// **Solvent-only is MOST characterisations, so the fire-on-change rule is
/// doing the real work here.** The water relation answers this beaker
/// every time the stack runs over it, with the same engine, the same file
/// and the same reason every time — so it is news exactly once.
#[test]
fn plain_water_says_which_relation_answered_it_exactly_once() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    let poured = bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new("water"),
                moles: Moles(55.51),
                at: None,
            },
            &mut eq,
            &PermissiveScreen,
        )
        .expect("step");
    let opening = routed(&poured, v);
    assert_eq!(
        opening.len(),
        1,
        "the water says where its own numbers come from: {poured:#?}"
    );
    assert!(
        opening[0].routing.contains("without invoking IPhreeqc"),
        "{}",
        opening[0].routing
    );
    assert_eq!(opening[0].dataset_file(), "phreeqc.dat");

    // More of the same water. The relation has not moved, so there is
    // nothing to say — this is the half that keeps the line off every
    // step of every lesson that starts with a beaker of water.
    let more = bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new("water"),
                moles: Moles(10.0),
                at: None,
            },
            &mut eq,
            &PermissiveScreen,
        )
        .expect("step");
    assert!(
        routed(&more, v).is_empty(),
        "the routing had not changed: {:#?}",
        routed(&more, v)
    );
}

/// And the beaker still carries the provenance in its state, which is what
/// `kero explain` reads.
#[test]
fn the_water_relation_is_still_recorded_on_the_vessel() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 55.51);
    let p = bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("characterised")
        .provenance
        .expect("provenance recorded");
    assert!(
        p.routing.contains("without invoking IPhreeqc"),
        "{}",
        p.routing
    );
}

#[test]
fn the_routing_is_announced_when_the_dataset_changes_under_the_beaker() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 55.51);

    // The first solve that actually routes to a dataset differs from
    // nothing, so it is news: the beaker has a source now and did not
    // before.
    let salted = bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new("NaCl"),
                moles: Moles(0.05),
                at: None,
            },
            &mut eq,
            &PermissiveScreen,
        )
        .expect("step");
    let opening = routed(&salted, v);
    assert_eq!(
        opening.len(),
        1,
        "the beaker says where its answers come from, once: {salted:#?}"
    );
    assert_eq!(opening[0].dataset, WATEQ_DATASET);

    // Enough salt to put the solution past where the default dataset is
    // reliable. That is a different FILE answering the same beaker, which
    // is exactly the sentence a learner should be shown at the moment it
    // becomes true.
    let brined = bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new("NaCl"),
                moles: Moles(8.0),
                at: None,
            },
            &mut eq,
            &PermissiveScreen,
        )
        .expect("step");
    let moved = routed(&brined, v);
    assert_eq!(
        moved.len(),
        1,
        "one event for one change of dataset: {moved:#?}"
    );
    assert_eq!(moved[0].dataset, PITZER_DATASET);
    assert_ne!(
        opening[0].dataset, moved[0].dataset,
        "the announcement is the change, not a repeat"
    );
    assert!(
        moved[0].model.contains("Pitzer"),
        "and it carries the model that came with the file: {}",
        moved[0].model
    );
    assert!(
        moved[0].routing.contains("concentrated"),
        "and the reason: {}",
        moved[0].routing
    );
}

#[test]
fn a_repeat_characterisation_on_the_same_source_says_nothing() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 55.51);
    // The routing is stated once, here.
    add(&mut bench, &mut eq, v, "NaCl", 0.05);

    // A second helping of the SAME solute on the same dataset, the same
    // model and the same reason. The solution is re-characterised — pH and
    // ionic strength both move, and `SolutionCharacterized` says so — and
    // the source has not moved at all.
    for round in 0..2 {
        let more = bench
            .step_with(
                Operator::Add {
                    vessel: v,
                    species: SpeciesId::new("NaCl"),
                    moles: Moles(0.05),
                    at: None,
                },
                &mut eq,
                &PermissiveScreen,
            )
            .expect("step");
        assert!(
            more.iter()
                .any(|e| matches!(e, Event::SolutionCharacterized { .. })),
            "round {round}: the beaker really was characterised again: {more:#?}"
        );
        assert!(
            routed(&more, v).is_empty(),
            "round {round}: the routing had not changed, so there is nothing to say: {:#?}",
            routed(&more, v)
        );
    }
}

/// The concentrated route's molality moves with every spoonful, and the
/// routing does not.
///
/// `routing.concentrated-ion-interaction` renders *chosen because the
/// solution is concentrated (~16.0 mol/kgw)*. Comparing the rendered
/// sentence would announce a routing change on every step of a lesson
/// that adds salt to brine. `Provenance::source_key` compares the recipe's
/// SHAPE, so the measurement is out and the reasons are in.
#[test]
fn a_number_moving_inside_a_reason_is_not_a_new_reason() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 55.51);
    add(&mut bench, &mut eq, v, "NaCl", 8.0);
    let before = bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .unwrap()
        .provenance
        .unwrap();

    let more = bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new("NaCl"),
                moles: Moles(1.0),
                at: None,
            },
            &mut eq,
            &PermissiveScreen,
        )
        .expect("step");
    let after = bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .unwrap()
        .provenance
        .unwrap();

    assert_ne!(
        before.routing, after.routing,
        "the sentence really did change — the molality in it moved"
    );
    assert!(
        before.same_source_as(&after),
        "and it is still the same routing:\n  {}\n  {}",
        before.routing,
        after.routing
    );
    assert!(
        routed(&more, v).is_empty(),
        "so nothing is announced: {:#?}",
        routed(&more, v)
    );
}
