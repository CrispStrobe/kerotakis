//! Cobalt(II) chloride in concentrated hydrochloric acid (2026-11-26).
//!
//! The solve refused outright: minteq.v4.dat, the route that carries
//! cobalt, also defines a cobalt(III) state (`Co+3 + e- = Co+2`, log K 32.4,
//! Bard 1985), and against the strongly acidic, chloride-rich solution its
//! pe split put more than 1e-10 mol into `Co(3)`, which the registry has no
//! ion for, so the readback refused the whole step as unrepresented. Cobalt
//! is outside `FAST_REDOX`: it keeps the oxidation state it was added in,
//! and that share now returns to the posed cobalt(II).
//!
//! Positive: CoCl2 in concentrated HCl solves, keeps every cobalt as Co+2,
//! and the chloro complex CoCl+ appears (since 2026-11-27 on sit.dat, via
//! the Co-Cl activity route, rather than minteq.v4.dat).
//! Negative: dilute CoCl2 in water stays essentially free Co2+.
#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

const COCL2_G_PER_MOL: f64 = 129.833;

fn run(script: &[&str]) -> (Bench, Vec<Vec<Event>>) {
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]));
    let mut steps = Vec::new();
    for op in script {
        let parsed = kerotakis_core::script::parse_op(op)
            .expect("parse")
            .expect("a known verb");
        steps.push(
            bench
                .step_with(parsed, &mut stack, &PermissiveScreen)
                .unwrap_or_else(|e| panic!("{op}: {e}\nearlier steps: {steps:#?}")),
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

fn cobalt_species(bench: &Bench) -> Vec<(String, f64)> {
    let info = bench
        .vessel(VesselId(0))
        .unwrap()
        .solution
        .clone()
        .expect("a cobalt chloride solution is characterised");
    info.species
        .iter()
        .filter(|s| s.name.starts_with("Co") && !s.name.starts_with("CO"))
        .map(|s| (s.name.clone(), s.molality))
        .collect()
}

fn refused_or_lost_cobalt(steps: &[Vec<Event>]) -> bool {
    steps.iter().flatten().any(|event| {
        let text = format!("{event:?}");
        text.contains("Co(3)") || text.contains("readback-unrepresented")
    })
}

#[test]
fn cobalt_chloride_in_concentrated_hydrochloric_acid_solves_as_chloro_cobalt_ii() {
    let (bench, steps) = run(&[
        "add v1 water 5mL",
        "add v1 CoCl2 0.5g",
        "add v1 hydrochloric_acid_37_percent 5mL",
        "wait 10s",
    ]);
    let n = 0.5 / COCL2_G_PER_MOL;
    let species = cobalt_species(&bench);
    eprintln!("cobalt speciation in conc. HCl: {species:?}");
    assert!(!refused_or_lost_cobalt(&steps), "{steps:#?}");
    // Every cobalt went in as cobalt(II) and is still booked as cobalt(II).
    let co = moles(&bench, "Co+2");
    assert!((co - n).abs() < 1e-6 * n, "Co+2 {co} vs {n}: {steps:#?}");
    assert!(moles(&bench, "CoCl2") < 1e-12);
    // At this chloride load the reviewed activity route
    // (`activity_routes`, Co-Cl) poses the solution on sit.dat, which
    // defines CoCl+ through CoCl4-2 with SIT activities; the solid CoCl2
    // (a phase sit.dat spells differently) is freed into the Co/Cl totals.
    // CoCl+ is asserted as the chloro complex every route carrying cobalt
    // defines; the higher complexes' shares are SIT extrapolations above
    // the route's ionic-strength ceiling and are not asserted here.
    let cocl = species
        .iter()
        .find(|(name, _)| name == "CoCl+")
        .map(|(_, m)| *m)
        .unwrap_or_else(|| panic!("CoCl+ in the speciation: {species:?}"));
    assert!(cocl > 0.0, "{species:?}");
    let ph = bench
        .vessel(VesselId(0))
        .unwrap()
        .solution
        .clone()
        .unwrap()
        .ph;
    assert!(ph < 1.0, "concentrated hydrochloric acid: pH {ph}");
}

#[test]
fn dilute_cobalt_chloride_stays_free_cobalt_ii() {
    let (bench, steps) = run(&["add v1 water 100mL", "add v1 CoCl2 0.05g", "wait 10s"]);
    let n = 0.05 / COCL2_G_PER_MOL;
    let species = cobalt_species(&bench);
    eprintln!("cobalt speciation, dilute: {species:?}");
    assert!(!refused_or_lost_cobalt(&steps), "{steps:#?}");
    let co = moles(&bench, "Co+2");
    assert!((co - n).abs() < 1e-6 * n, "Co+2 {co} vs {n}");
    let total: f64 = species.iter().map(|(_, m)| m).sum();
    let free = species
        .iter()
        .find(|(name, _)| name == "Co+2")
        .map(|(_, m)| *m)
        .unwrap_or_else(|| panic!("free Co+2 in the speciation: {species:?}"));
    assert!(
        free / total > 0.9,
        "free Co2+ fraction {}: {species:?}",
        free / total
    );
    let chloro: f64 = species
        .iter()
        .filter(|(name, _)| name.starts_with("CoCl"))
        .map(|(_, m)| m)
        .sum();
    assert!(
        chloro / total < 0.05,
        "dilute chloride complexes cobalt: {species:?}"
    );
}
