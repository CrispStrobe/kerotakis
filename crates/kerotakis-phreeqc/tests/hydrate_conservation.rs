//! A dissolved hydrate keeps every atom it brought.
//!
//! The bug this pins: a hydrate whose mineral phase the routed database
//! does not define (bischofite, MgCl2·6H2O, is written only in pitzer.dat
//! and a dilute beaker routes to wateq4f) was kept as a phase `build_input`
//! could not name, silently skipped, read back at zero and reported as
//! dissolved — and its magnesium and chloride were gone from the vessel on
//! the next step. The engine-free side of the fix is pinned beside
//! `setup_problem` in `aqueous.rs`; this is the whole-bench side: for every
//! registry hydrate the engine owns, the non-H/O elements in the vessel stay
//! exactly what was added across dissolution and two further steps.
#![cfg(feature = "engine")]

use std::collections::BTreeMap;

use kerotakis_core::*;
use kerotakis_phreeqc::derived::{self, DerivedRole};
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn elements_of(key: &str) -> BTreeMap<String, f64> {
    let data = species::lookup_key(key).unwrap_or_else(|| panic!("{key} in registry"));
    stoich::parse_formula(data.formula)
        .unwrap_or_else(|e| panic!("{key}: {e:?}"))
        .counts
        .into_iter()
        .filter(|(el, _)| el != "H" && el != "O")
        .collect()
}

/// Non-H/O element inventory of the whole vessel, every phase.
fn inventory(bench: &Bench) -> BTreeMap<String, f64> {
    let mut out = BTreeMap::new();
    for p in &bench.vessel(VesselId(0)).unwrap().contents {
        if species::is_acid_base_basis(&p.species.0) || p.species.0 == "water" {
            continue;
        }
        for (el, n) in elements_of(&p.species.0) {
            *out.entry(el).or_insert(0.0) += n * p.moles.0;
        }
    }
    out
}

fn liquid(bench: &Bench) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .contents
        .iter()
        .filter(|p| p.species.0 == "water" && p.phase == Phase::Liquid)
        .map(|p| p.moles.0)
        .sum()
}

/// Every registry hydrate the aqueous engine owns as a mineral.
fn engine_hydrates() -> Vec<&'static str> {
    species::REGISTRY
        .iter()
        .filter(|data| {
            matches!(derived::role(data.key), Some(DerivedRole::Mineral { phase, .. })
                if derived::phase_by_name(phase).is_some_and(|p| p.waters > 0.0))
        })
        .map(|data| data.key)
        .collect()
}

#[test]
fn every_engine_hydrate_conserves_its_elements_across_steps() {
    let hydrates = engine_hydrates();
    for expected in ["chalcanthite", "epsomite", "gypsum"] {
        assert!(
            hydrates.contains(&expected),
            "{expected} is an engine hydrate"
        );
    }
    for key in hydrates {
        let mut bench = Bench::new();
        let mut stack = SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
            PhreeqcEquilibrator::new().expect("engine"),
        )]));
        let mut step = |bench: &mut Bench, op: &str| {
            bench
                .step_with(
                    kerotakis_core::script::parse_op(op)
                        .expect("parse")
                        .expect("a known verb"),
                    &mut stack,
                    &PermissiveScreen,
                )
                .unwrap_or_else(|e| panic!("{key}: {op}: {e:?}"))
        };
        step(&mut bench, "add v1 water 100mL");
        let data = species::lookup_key(key).unwrap();
        // A small dose: dissolves where it can, and stays honest where it
        // cannot (gypsum's solubility is the limit, not this test).
        let grams = 0.1;
        let n = grams / data.molar_mass;
        let added = step(&mut bench, &format!("add v1 {key} {grams}g"));
        let want: BTreeMap<String, f64> = elements_of(key)
            .into_iter()
            .map(|(el, c)| (el, c * n))
            .collect();
        let mut trail = vec![added];
        for op in ["wait 10s", "wait 10s"] {
            trail.push(step(&mut bench, op));
            let have = inventory(&bench);
            for (el, amount) in &want {
                let got = have.get(el).copied().unwrap_or(0.0);
                assert!(
                    (got - amount).abs() <= 1e-9 * amount,
                    "{key}: {el} {got} vs added {amount} after {op}: {trail:#?}"
                );
            }
        }
    }
}

/// The crystal water is not lost either: a hydrate that dissolves
/// completely adds its waters to the liquid.
#[test]
fn a_dissolved_hydrate_releases_its_crystal_water() {
    for (key, waters) in [("chalcanthite", 5.0), ("epsomite", 7.0)] {
        let mut bench = Bench::new();
        let mut stack = SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
            PhreeqcEquilibrator::new().expect("engine"),
        )]));
        let mut run = |bench: &mut Bench, op: &str| {
            bench
                .step_with(
                    kerotakis_core::script::parse_op(op).unwrap().unwrap(),
                    &mut stack,
                    &PermissiveScreen,
                )
                .unwrap()
        };
        run(&mut bench, "add v1 water 100mL");
        let water0 = liquid(&bench);
        run(&mut bench, &format!("add v1 {key} 1g"));
        run(&mut bench, "wait 10s");
        let n = 1.0 / species::lookup_key(key).unwrap().molar_mass;
        let gained = liquid(&bench) - water0;
        assert!(
            (gained - waters * n).abs() < 0.02 * waters * n + 1e-3,
            "{key}: liquid water grew by {gained} mol, crystal water {}",
            waters * n
        );
    }
}
