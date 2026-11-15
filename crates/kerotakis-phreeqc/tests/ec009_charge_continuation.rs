//! One unchanged original titration on the authenticated pre-titration state.
//! No preparation-prefix replay and no new chemistry rule.
#![cfg(feature = "engine")]
use kerotakis_core::{
    ledger::ConservedLedger,
    script::{parse_command, parse_op, Command},
    *,
};
use kerotakis_phreeqc::PhreeqcEquilibrator;
use std::io::Write;
fn emit(name: &str, value: &serde_json::Value) {
    if let Some(dir) = std::env::var_os("KEROTAKIS_EC009_EVIDENCE_DIR") {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(std::path::PathBuf::from(dir).join(name))
            .unwrap();
        serde_json::to_writer_pretty(&mut f, value).unwrap();
        f.write_all(b"\n").unwrap();
    }
}

/// A shell observation is a canonical frontend command, not a fabricated
/// bench operator. Only `inspect` takes this read-only branch; other shell
/// commands must not be silently accepted by a scientific fixture.
fn observation(
    bench: &mut Bench,
    eq: &mut dyn Equilibrator,
    text: &str,
) -> Result<(Command, Vec<Event>), String> {
    let command = parse_command(text, Locale::EN).map_err(|error| format!("{error:?}"))?;
    let events = match command.operator.as_ref() {
        Some(op) => bench
            .step_with(op.clone(), eq, &PermissiveScreen)
            .map_err(|error| format!("{error:?}"))?,
        None if command.canonical == "inspect" => Vec::new(),
        None => {
            return Err(format!(
                "unsupported fixture command: {}",
                command.canonical
            ))
        }
    };
    Ok((command, events))
}

fn redox_refusals(events: &[Event]) -> Vec<&Event> {
    events
        .iter()
        .filter(|event| {
            matches!(event, Event::NotYetModeled { reason: Some(reason), .. }
                if reason.key == "not-modeled.redox-coupling-stood-down")
        })
        .collect()
}
#[test]
fn unchanged_ec009_titration_preserves_the_prepared_atoms_and_charge() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/ec009-before-titration-20261114.json"
    ))
    .unwrap();
    let mut bench = Bench::new();
    bench.vessels = serde_json::from_value(fixture["before_titration"].clone()).unwrap();
    let mut stack = kerotakis_stack::standard_stack(vec![Box::new(
        PhreeqcEquilibrator::new().expect("native engine"),
    )]);
    let mut preparation = bench.vessels[0].clone();
    let original = parse_op("titrate v1 KMnO4 0.02M 0.1mL until colour persists max 200")
        .unwrap()
        .unwrap();
    assert_eq!(
        serde_json::to_value(&original).unwrap(),
        fixture["original_titration"]
    );
    let before = serde_json::to_value(&bench.vessels).unwrap();
    let result = bench.step_with(original.clone(), &mut stack, &PermissiveScreen);
    let events = result.as_ref().ok().cloned().unwrap_or_default();
    emit(
        "titration.json",
        &serde_json::json!({
            "physical_source":fixture["physical_source"],"historical_native_binary_sha256":null,
            "stage":"Changed-source native titration continuation only; preparation prefix was not rerun",
            "operator":original,"before":before,"after":serde_json::to_value(&bench.vessels).unwrap(),
            "events":events,"error":result.as_ref().err().map(|e|format!("{e:?}"))
        }),
    );
    result.expect("unchanged original titration continuation");
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })),
        "{events:?}"
    );
    let (volume, concentration) = events
        .iter()
        .find_map(|e| match e {
            Event::Titrated {
                total_volume,
                concentration,
                ..
            } => Some((*total_volume, *concentration)),
            _ => None,
        })
        .expect("actual burette result");
    preparation.deposit(
        SpeciesId::new("KMnO4"),
        Moles(volume.0 * concentration),
        Phase::Solid,
    );
    preparation.deposit(
        SpeciesId::new("water"),
        species::lookup(&SpeciesId::new("water"))
            .unwrap()
            .moles_from_liters(volume),
        Phase::Liquid,
    );
    let expected = ConservedLedger::from_vessel(&preparation);
    let actual = ConservedLedger::from_vessel(&bench.vessels[0]);
    assert!(
        (expected.charge - actual.charge).abs() <= 1e-10,
        "charge {} -> {}",
        expected.charge,
        actual.charge
    );
    for (element, moles) in expected.elements {
        assert!(
            (actual.elements.get(&element).copied().unwrap_or(0.0) - moles).abs() <= 1e-10,
            "{element}"
        );
    }
    let v = &bench.vessels[0];
    assert!(v.moles_of(&SpeciesId::new("Fe+3")).0 > 0.0009);
    assert!(v.moles_of(&SpeciesId::new("Fe+2")).0 < 0.0001);
    assert!(v.moles_of(&SpeciesId::new("Mn+2")).0 > 0.00018);
    assert_eq!(v.moles_of(&SpeciesId::new("MnO2")).0, 0.0);
    for (text, name) in [
        ("measure v1 uvvis", "spectrum.json"),
        ("inspect", "inspect.json"),
    ] {
        let before = serde_json::to_value(&bench.vessels).unwrap();
        let (command, events) = observation(&mut bench, &mut stack, text).unwrap();
        let after = serde_json::to_value(&bench.vessels).unwrap();
        emit(
            name,
            &serde_json::json!({"canonical_command":command.canonical,"operator":command.operator,"events":events,"before":before,"after":after}),
        );
        assert_eq!(before, after);
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })));
    }
    // The historical stage predicates can pass after a refused last dose.
    // Complete observations first, then keep that actual model boundary a
    // failure: a closed fallback is not a solved chemical endpoint.
    let refusals = redox_refusals(&events);
    assert!(
        refusals.is_empty(),
        "terminal redox coupling is unsupported: {refusals:?}"
    );
}

struct NoObservationSolver;
impl Equilibrator for NoObservationSolver {
    fn name(&self) -> &'static str {
        "inspection must not query a solver"
    }
    fn equilibrate(&mut self, _: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        panic!("frontend inspection queried a chemical solver")
    }
}

#[test]
fn inspect_is_a_canonical_frontend_observation_without_a_bench_operator() {
    let mut bench = Bench::new();
    let before = serde_json::to_value(&bench.vessels).unwrap();
    let (command, events) = observation(&mut bench, &mut NoObservationSolver, "inspect").unwrap();
    assert_eq!(command.canonical, "inspect");
    assert!(command.operator.is_none());
    assert!(events.is_empty());
    assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
}

#[test]
fn unrelated_frontend_commands_cannot_masquerade_as_inspection() {
    for text in ["register lv3", "help", "explain"] {
        assert!(observation(&mut Bench::new(), &mut NoObservationSolver, text).is_err());
    }
}

#[test]
fn saved_last_state_reports_the_refusal_without_replaying_the_titration() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/ec009-after-first-precision-20261115.json"
    ))
    .unwrap();
    let mut vessel: Vessel = serde_json::from_value(fixture["after"][0].clone()).unwrap();
    let before = serde_json::to_value(&vessel).unwrap();
    let before_ledger = ConservedLedger::from_vessel(&vessel);
    let mut eq = PhreeqcEquilibrator::new().unwrap();
    let result = eq.equilibrate(&mut vessel);
    emit(
        "last-state-diagnostic.json",
        &serde_json::json!({"saved_source":fixture["source"],"saved_titration_sha256":fixture["titration_sha256"],
            "stage":"one characterization of the exact saved final state; no dose, titration or preparation replay",
            "before":before,"after":serde_json::to_value(&vessel).unwrap(),
            "events":result.as_ref().ok(),"error":result.as_ref().err().map(|error|format!("{error:?}"))}),
    );
    let events = result.expect("characterization must retain a typed fallback");
    assert!(!redox_refusals(&events).is_empty());
    let after_ledger = ConservedLedger::from_vessel(&vessel);
    assert!((after_ledger.charge - before_ledger.charge).abs() <= 1e-10);
    for (element, amount) in before_ledger.elements {
        assert!(
            (after_ledger.elements.get(&element).copied().unwrap_or(0.0) - amount).abs() <= 1e-10,
            "{element}"
        );
    }
}
