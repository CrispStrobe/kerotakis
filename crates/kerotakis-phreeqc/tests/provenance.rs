//! Provenance and multiple paths: every answer says where it came from,
//! and the same question can be put to every dataset that can express it
//! (PLAN.md: be open about sources, offer the paths rather than asserting
//! one).

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::{PathOutcome, PhreeqcEquilibrator};

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
    assert_eq!(p.dataset, "wateq4f.dat");
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
    assert_eq!(p.dataset, "pitzer.dat");
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
        .find(|p| p.dataset == "pitzer.dat")
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
