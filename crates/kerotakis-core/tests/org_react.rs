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

/// Element inventory of the whole vessel, in moles of each element.
/// Stronger than a mass sum and blind to phase: a route that turned a
/// carbon into the same mass of hydrogen would pass a mass check and fails
/// this one, and a product that dissolves or degasses inside the vessel
/// moves neither total.
fn elements(bench: &Bench) -> std::collections::BTreeMap<String, f64> {
    let mut totals = std::collections::BTreeMap::<String, f64>::new();
    for portion in &bench.vessel(VesselId(0)).unwrap().contents {
        let Some(data) = species::lookup(&portion.species) else {
            continue;
        };
        let parsed = stoich::parse_formula(data.formula).expect("registry formula parses");
        for (element, count) in &parsed.counts {
            *totals.entry(element.clone()).or_default() += portion.moles.0 * count;
        }
    }
    totals
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

/// The atom balance is what stands in for the SMIRKS template on the two
/// rows that have none. Checking the MASS is not the same thing — a row
/// that lost a carbon and gained the same mass in hydrogen would pass a
/// mass check — so this counts elements, and it is deliberately a totality
/// test over the whole table rather than a per-row one.
#[test]
fn every_named_reaction_balances_its_atoms_and_its_charge() {
    use std::collections::BTreeMap;

    fn tally(side: &[(&str, f64)]) -> (BTreeMap<String, f64>, f64) {
        let mut counts = BTreeMap::<String, f64>::new();
        let mut charge = 0.0;
        for (key, coeff) in side {
            let data = species::lookup(&SpeciesId::new(key))
                .unwrap_or_else(|| panic!("{key} is a registry species"));
            let parsed = stoich::parse_formula(data.formula)
                .unwrap_or_else(|e| panic!("{key} formula parses: {e}"));
            for (element, count) in &parsed.counts {
                *counts.entry(element.clone()).or_default() += coeff * count;
            }
            charge += coeff * parsed.charge;
        }
        (counts, charge)
    }

    for reaction in curated::ORG_REACTIONS {
        let (into, charge_in) = tally(reaction.reactants);
        let out_of: Vec<(&str, f64)> = reaction
            .products
            .iter()
            .map(|(key, coeff, _)| (*key, *coeff))
            .collect();
        let (out, charge_out) = tally(&out_of);
        assert_eq!(
            into.keys().collect::<Vec<_>>(),
            out.keys().collect::<Vec<_>>(),
            "{}: different elements on the two sides",
            reaction.name
        );
        for (element, count) in &into {
            assert!(
                (count - out[element]).abs() < 1e-9,
                "{}: {element} is {count} in and {} out",
                reaction.name,
                out[element]
            );
        }
        assert!(
            (charge_in - charge_out).abs() < 1e-9,
            "{}: charge {charge_in} in, {charge_out} out",
            reaction.name
        );
    }
}

/// bio-064. The oxygen limits, the vinegar appears, and the mass is the
/// mass that went in.
#[test]
fn alcohol_oxidation_makes_the_acid_and_the_oxygen_limits() {
    let mut bench = Bench::new();
    add(&mut bench, "ethanol", 0.10);
    add(&mut bench, "O2", 0.04);
    let before = elements(&bench);
    let events = bench
        .step(
            script::parse_op("react v1 alcohol-oxidation")
                .expect("grammar")
                .expect("an operator"),
        )
        .expect("react");
    let extent = events
        .iter()
        .find_map(|e| match e {
            Event::OrgReacted { extent, name, .. } if name == "alcohol-oxidation" => Some(extent.0),
            _ => None,
        })
        .expect("an OrgReacted event");
    assert!(
        (extent - 0.04).abs() < 1e-12,
        "the oxygen limits: extent {extent}"
    );
    let v = bench.vessel(VesselId(0)).unwrap();
    let moles_of = |key: &str| v.moles_of(&SpeciesId::new(key)).0;
    assert!((moles_of("CH3COOH") - 0.04).abs() < 1e-12);
    assert!((moles_of("water") - 0.04).abs() < 1e-12);
    assert!(
        (moles_of("ethanol") - 0.06).abs() < 1e-12,
        "the unoxidised alcohol stays"
    );
    let after = elements(&bench);
    assert_eq!(
        before.keys().collect::<Vec<_>>(),
        after.keys().collect::<Vec<_>>()
    );
    for (element, count) in &before {
        assert!(
            (count - after[element]).abs() < 1e-9,
            "{element}: {count} mol in, {} mol out",
            after[element]
        );
    }
}

/// The differential that keeps two routes to vinegar from disagreeing.
/// `fermentation.rs` runs this same oxidation for the acetic-acid bacteria
/// culture; if either side's coefficients are edited, this fails.
#[test]
fn the_named_oxidation_is_the_culture_s_oxidation() {
    let row = curated::ORG_REACTIONS
        .iter()
        .find(|r| r.name == "alcohol-oxidation")
        .expect("the row exists");
    assert_eq!(
        row.reactants.to_vec(),
        vec![("ethanol", 1.0), ("O2", 1.0)],
        "the culture consumes one ethanol and one oxygen"
    );
    assert_eq!(
        row.products
            .iter()
            .map(|(key, coeff, _)| (*key, *coeff))
            .collect::<Vec<_>>(),
        vec![("CH3COOH", 1.0), ("water", 1.0)],
        "the fermentation lane's acetic route deposits one acid and one \
         water per ethanol; these must not drift apart"
    );
}

/// bio-080. Six oxygens per sugar, six gases and six waters out, and the
/// beaker no warmer than it was — the enthalpy is quoted, not applied.
#[test]
fn respiration_consumes_six_oxygens_and_warms_nothing() {
    let mut bench = Bench::new();
    add(&mut bench, "glucose", 0.10);
    add(&mut bench, "O2", 0.60);
    let before = elements(&bench);
    let temperature_before = bench.vessel(VesselId(0)).unwrap().temperature;
    let events = bench
        .step(
            script::parse_op("react v1 respiration")
                .expect("grammar")
                .expect("an operator"),
        )
        .expect("react");
    let extent = events
        .iter()
        .find_map(|e| match e {
            Event::OrgReacted { extent, name, .. } if name == "respiration" => Some(extent.0),
            _ => None,
        })
        .expect("an OrgReacted event");
    assert!(
        (extent - 0.10).abs() < 1e-12,
        "the sugar limits at exactly six oxygens each: extent {extent}"
    );
    let v = bench.vessel(VesselId(0)).unwrap();
    let moles_of = |key: &str| v.moles_of(&SpeciesId::new(key)).0;
    assert!(moles_of("glucose") < 1e-12, "the sugar is spent");
    assert!(moles_of("O2") < 1e-12, "so is the oxygen");
    assert!((moles_of("CO2") - 0.60).abs() < 1e-12);
    assert!((moles_of("water") - 0.60).abs() < 1e-12);
    let after = elements(&bench);
    assert_eq!(
        before.keys().collect::<Vec<_>>(),
        after.keys().collect::<Vec<_>>()
    );
    for (element, count) in &before {
        assert!(
            (count - after[element]).abs() < 1e-9,
            "{element}: {count} mol in, {} mol out",
            after[element]
        );
    }
    assert!(
        (v.temperature.0 - temperature_before.0).abs() < 1e-9,
        "no reaction enthalpy is curated for this table, so -2803 kJ/mol is \
         quoted in the boundary and NOT applied: {} K -> {} K",
        temperature_before.0,
        v.temperature.0
    );
}
