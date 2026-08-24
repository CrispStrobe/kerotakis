//! The react verb: deliberate curated organic transformations, exactly
//! conservative, honest when they have nothing to work on.

use kerotakis_core::*;

fn mass_g(bench: &Bench) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .contents
        .iter()
        .filter_map(|p| species::lookup(&p.species).map(|d| p.moles.0 * d.molar_mass))
        .sum()
}

fn add(bench: &mut Bench, key: &str, moles: f64) {
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(key),
            moles: Moles(moles),
            at: None,
        })
        .expect("add");
}

#[test]
fn esterification_makes_the_ester_and_conserves_mass() {
    let mut bench = Bench::new();
    add(&mut bench, "CH3COOH", 0.10);
    add(&mut bench, "ethanol", 0.15);
    let before = mass_g(&bench);
    let events = bench
        .step(
            script::parse_op("react v1 esterification")
                .expect("grammar")
                .expect("an operator"),
        )
        .expect("react");
    let extent = events
        .iter()
        .find_map(|e| match e {
            Event::OrgReacted { extent, name, .. } if name == "esterification" => Some(extent.0),
            _ => None,
        })
        .expect("an OrgReacted event");
    assert!(
        (extent - 0.10).abs() < 1e-12,
        "the acid limits: extent {extent}"
    );
    let v = bench.vessel(VesselId(0)).unwrap();
    let moles_of = |key: &str| v.moles_of(&SpeciesId::new(key)).0;
    assert!((moles_of("ethyl_acetate") - 0.10).abs() < 1e-12);
    assert!((moles_of("water") - 0.10).abs() < 1e-12);
    assert!(moles_of("CH3COOH") < 1e-12, "the acid is spent");
    assert!(
        (moles_of("ethanol") - 0.05).abs() < 1e-12,
        "the excess alcohol stays"
    );
    let after = mass_g(&bench);
    assert!(
        (after - before).abs() < 1e-9,
        "mass conserved: {before} g -> {after} g"
    );
}

#[test]
fn the_round_trip_returns_the_alcohol() {
    let mut bench = Bench::new();
    add(&mut bench, "CH3COOH", 0.10);
    add(&mut bench, "ethanol", 0.10);
    bench
        .step(
            script::parse_op("react v1 esterification")
                .unwrap()
                .unwrap(),
        )
        .unwrap();
    add(&mut bench, "NaOH", 0.10);
    let events = bench
        .step(
            script::parse_op("react v1 saponification")
                .unwrap()
                .unwrap(),
        )
        .unwrap();
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::OrgReacted { name, .. } if name == "saponification")));
    let v = bench.vessel(VesselId(0)).unwrap();
    let moles_of = |key: &str| v.moles_of(&SpeciesId::new(key)).0;
    assert!(
        (moles_of("ethanol") - 0.10).abs() < 1e-12,
        "the alcohol came back"
    );
    assert!((moles_of("NaOAc") - 0.10).abs() < 1e-12, "the acid's salt");
    assert!(moles_of("ethyl_acetate") < 1e-12, "the ester is gone");
}

#[test]
fn a_missing_reactant_refuses_out_loud() {
    let mut bench = Bench::new();
    add(&mut bench, "ethanol", 0.10);
    let events = bench
        .step(
            script::parse_op("react v1 esterification")
                .unwrap()
                .unwrap(),
        )
        .expect("the step runs");
    assert!(
        !events.iter().any(|e| matches!(e, Event::OrgReacted { .. })),
        "nothing reacted"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("CH3COOH")
        )),
        "the refusal names what is missing"
    );
}

#[test]
fn an_unknown_reaction_fails_at_parse_time_with_the_shelf() {
    let err = script::parse_op("react v1 transmutation").unwrap_err();
    assert!(
        err.contains("esterification") && err.contains("saponification"),
        "the parse error lists the curated shelf, got: {err}"
    );
}
