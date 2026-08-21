//! The expert register's raw material: true equilibrium speciation with
//! activity coefficients, parsed from the engine's own report.

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

#[test]
fn speciation_shows_complexes_and_activity_coefficients() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 5.55);
    add(&mut bench, &mut eq, v, "NaCl", 0.01);
    add(&mut bench, &mut eq, v, "AgNO3", 0.01);

    let info = bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("characterised");
    let find = |name: &str| info.species.iter().find(|s| s.name == name);

    // The free ions are there with realistic activity coefficients (<1 for
    // charged species at I ~ 0.1 m).
    let na = find("Na+").expect("Na+ in speciation");
    let gamma_na = na.activity / na.molality;
    assert!(
        gamma_na > 0.7 && gamma_na < 0.9,
        "γ(Na+) ≈ 0.78, got {gamma_na}"
    );

    // The neutral complex — the thing a lookup table can never show.
    assert!(find("AgCl").is_some(), "AgCl(aq) complex must appear");

    // No duplicates (species are listed under every element section in the
    // raw report).
    let mut names: Vec<&str> = info.species.iter().map(|s| s.name.as_str()).collect();
    let before = names.len();
    names.dedup();
    names.sort_unstable();
    names.dedup();
    assert_eq!(before, names.len(), "speciation must be deduplicated");

    // Descending by molality.
    for pair in info.species.windows(2) {
        assert!(pair[0].molality >= pair[1].molality);
    }
}

#[test]
fn speciation_is_cached_with_the_result() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let run = |eq: &mut PhreeqcEquilibrator| {
        let mut bench = Bench::new();
        let v = VesselId(0);
        add(&mut bench, eq, v, "water", 5.55);
        add(&mut bench, eq, v, "NaCl", 0.02);
        bench.vessel(v).unwrap().solution.clone().unwrap().species
    };
    let first = run(&mut eq);
    let second = run(&mut eq);
    assert!(eq.cache_hits() > 0);
    assert_eq!(first, second, "cached speciation is identical");
    assert!(!first.is_empty());
}
