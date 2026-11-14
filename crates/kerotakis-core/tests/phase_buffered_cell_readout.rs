//! Source-exact continuation of authenticated native EC004 readout states.
//! These checks run no native solver and do not replay the chemical prefix.
use kerotakis_core::{displacement, script::parse_op, *};
use std::io::Write;

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "fixtures/ec004-native-readout-states-20261114.json"
    ))
    .unwrap()
}
fn states(key: &str) -> Vec<Vessel> {
    serde_json::from_value(fixture()[key].clone()).unwrap()
}
fn emit(name: &str, value: &serde_json::Value) {
    if let Some(directory) = std::env::var_os("KEROTAKIS_READOUT_EVIDENCE_DIR") {
        let path = std::path::PathBuf::from(directory).join(name);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .unwrap();
        serde_json::to_writer_pretty(&mut file, value).unwrap();
        file.write_all(b"\n").unwrap();
    }
}
fn free(vessel: &Vessel, species: &str) -> f64 {
    vessel
        .resolved_aqueous_activity(&SpeciesId::new(species))
        .unwrap()
}
fn record(key: &str) -> (Vec<Event>, Bench) {
    let mut bench = Bench::new();
    bench.vessels = states(key);
    let before = serde_json::to_value(&bench.vessels).unwrap();
    let operator = parse_op("cell v1 v2").unwrap().unwrap();
    let events = bench.step(operator.clone()).unwrap();
    let after = serde_json::to_value(&bench.vessels).unwrap();
    assert_eq!(
        before, after,
        "a readout cannot replace or consume native stock"
    );
    emit(
        &format!("{key}.json"),
        &serde_json::json!({
            "operator": operator, "events": events, "before": before, "after": after,
            "physical_source": fixture()["physical_source"],
            "historical_native_binary_sha256": null,
            "stage": "Changed-source core readout only; historical native chemical prefix was not rerun"
        }),
    );
    (events, bench)
}

#[test]
fn unchanged_original_cell_operations_read_zero_then_native_silver_activity_without_mutation() {
    let (first, _) = record("before_chloride");
    assert!(first
        .iter()
        .any(|event| matches!(event, Event::NoCell { why, .. } if why.contains("0 V"))));
    assert!(!first
        .iter()
        .any(|event| matches!(event, Event::CellVoltage { .. })));
    let (events, bench) = record("after_chloride");
    let (volts, anode, cathode, standard) = events
        .iter()
        .find_map(|event| match event {
            Event::CellVoltage {
                volts,
                anode,
                cathode,
                standard_volts,
                ..
            } => Some((*volts, *anode, *cathode, *standard_volts)),
            _ => None,
        })
        .expect("finite native silver activity must remain readable beside AgCl");
    // The captured half-cells have slightly different actual temperatures;
    // use both, rather than imposing a common 25 C or a common slope.
    let expected = relations::nernst_slope(bench.vessels[0].temperature)
        * free(&bench.vessels[0], "Ag+").log10()
        - relations::nernst_slope(bench.vessels[1].temperature)
            * free(&bench.vessels[1], "Ag+").log10();
    assert!((volts - expected).abs() < 1e-12);
    assert!((0.1..0.3).contains(&volts));
    assert_eq!((anode, cathode), (VesselId(1), VesselId(0)));
    assert_eq!(
        standard, 0.0,
        "the same Ag+/Ag couple is used in both vessels"
    );
}

#[test]
fn explicit_leads_reverse_while_legacy_cell_magnitude_stays_positive() {
    let mut bench = Bench::new();
    bench.vessels = states("after_chloride");
    let before = serde_json::to_value(&bench.vessels).unwrap();
    let mut readings = Vec::new();
    for (positive, negative) in [(0, 1), (1, 0)] {
        let events = bench
            .step(Operator::Voltmeter {
                positive: VesselId(positive),
                negative: VesselId(negative),
            })
            .unwrap();
        readings.push(
            events
                .iter()
                .find_map(|event| match event {
                    Event::VoltmeterRead { volts, .. } => Some(*volts),
                    _ => None,
                })
                .unwrap(),
        );
    }
    assert!(readings[0] > 0.0);
    assert_eq!(readings[0], -readings[1]);
    let a = displacement::cell(&bench.vessels[0], &bench.vessels[1]).unwrap();
    let b = displacement::cell(&bench.vessels[1], &bench.vessels[0]).unwrap();
    assert_eq!(a.volts, b.volts);
    assert!(a.volts > 0.0);
    assert_ne!(a.anode_is_first, b.anode_is_first);
    assert_eq!(before, serde_json::to_value(&bench.vessels).unwrap());
    emit(
        "signed-leads.json",
        &serde_json::json!({"readings":readings,"before":before,"after":serde_json::to_value(&bench.vessels).unwrap()}),
    );
}

#[test]
fn readout_does_not_relax_reaction_electrolysis_or_pe_pinning_admission() {
    let pair = states("after_chloride");
    let mut low = pair[1].clone();
    assert!(low.moles_of(&SpeciesId::new("Ag+")).0 < OBSERVABLE_MOLES);
    let before = serde_json::to_value(&low).unwrap();
    assert!(displacement::electrode(&low).is_none());
    assert!(displacement::electrolyse(&low, 0.01, 1.0).is_none());
    assert!(displacement::pin_electrode(&mut low).is_none());
    assert!(displacement::cell(&pair[0], &low).is_ok());
    assert!(displacement::electrode(&low).is_none());
    assert_eq!(before, serde_json::to_value(&low).unwrap());
}

#[test]
fn unbuffered_trace_keeps_the_displacement_equivalence_refusal() {
    let pair = states("after_chloride");
    let mut low = pair[1].clone();
    low.contents.retain(|portion| portion.species.0 != "AgCl");
    assert!(displacement::cell(&pair[0], &low).is_err());
    assert!(displacement::electrode(&low).is_none());
}

#[test]
fn incomplete_unowned_nonfinite_and_ambiguous_states_have_no_extended_readout() {
    let pair = states("after_chloride");
    let mut outcomes = Vec::new();
    for invalid in 0..27 {
        let mut bad = pair[1].clone();
        match invalid {
            0 => bad.contents.retain(|p| p.species.0 != "AgCl"),
            1 => bad.contents.iter_mut().find(|p| p.species.0 == "AgCl").unwrap().phase = Phase::Aqueous,
            2 => bad.contents.retain(|p| p.species.0 != "Ag"),
            3 => bad.contents.iter_mut().find(|p| p.species.0 == "Ag").unwrap().phase = Phase::Liquid,
            4 => bad.contents.retain(|p| p.species.0 != "Ag+"),
            5 => bad.contents.iter_mut().find(|p| p.species.0 == "Ag+").unwrap().phase = Phase::Gas,
            6 => bad.contents.retain(|p| p.species.0 != "Cl-"),
            7 => bad.solution = None,
            8 => bad.solution.as_mut().unwrap().scope = vessel::SolutionScope::SolventOnly,
            9 => bad.solution.as_mut().unwrap().species.retain(|p| p.name != "Ag+"),
            10 => bad.solution.as_mut().unwrap().species.iter_mut().find(|p| p.name == "Ag+").unwrap().activity = 0.0,
            11 => bad.solution.as_mut().unwrap().species.iter_mut().find(|p| p.name == "Ag+").unwrap().activity = f64::NAN,
            12 => bad.solution.as_mut().unwrap().species.iter_mut().find(|p| p.name == "Ag+").unwrap().activity = f64::INFINITY,
            13 => bad.solution.as_mut().unwrap().species.iter_mut().find(|p| p.name == "Ag+").unwrap().molality = -1.0,
            14 => bad.solution.as_mut().unwrap().species.retain(|p| p.name != "Cl-"),
            15 => bad.solution.as_mut().unwrap().species.iter_mut().find(|p| p.name == "Cl-").unwrap().activity = 0.0,
            16 => bad.temperature = Kelvin(f64::NAN),
            17 => bad.pending_co2_transfer_mol = 1e-6,
            18 => bad.deposit(SpeciesId::new("Zn"), Moles(0.001), Phase::Solid),
            19 => bad.deposit(SpeciesId::new("unknown_readout_stock"), Moles(0.001), Phase::Solid),
            20 => bad.solution.as_mut().unwrap().solvent_kg = None,
            21 => bad.solution.as_mut().unwrap().solvent_kg = Some(0.2),
            22 => bad.contents.iter_mut().find(|p| p.species.0 == "Ag+").unwrap().moles.0 = -1e-8,
            23 => bad.solution.as_mut().unwrap().species.iter_mut().find(|p| p.name == "Ag+").unwrap().molality = 0.1,
            24 => bad.contents.retain(|p| p.species.0 != "water"),
            25 => bad.contents.iter_mut().find(|p| p.species.0 == "Ag").unwrap().moles.0 = f64::NAN,
            _ => bad.unresolved_materials.push(serde_json::from_value(serde_json::json!({"material":"opaque","recipe_id":"unknown","recipe_version":1,"basis":"mass_fraction","amount":0.1})).unwrap()),
        }
        let before = format!("{bad:?}");
        assert!(
            displacement::cell(&pair[0], &bad).is_err(),
            "invalid state {invalid} received an extended readout"
        );
        assert_eq!(format!("{bad:?}"), before);
        outcomes.push(serde_json::json!({"case":invalid,"refused":true,"unchanged":true}));
    }
    emit("negative-cases.json", &serde_json::json!(outcomes));
}
