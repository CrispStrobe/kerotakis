//! Native controls for the electron boundary that earns a reported pe.
//! The source-unit matrix checks pose/metadata; these controls use actual
//! IPhreeqc state and preserve the existing physical redox tests.
#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn step<E: Equilibrator>(bench: &mut Bench, solver: &mut E, op: Operator) {
    let events = bench
        .step_with(op, solver, &PermissiveScreen)
        .expect("native step");
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::SolverFailed { .. })),
        "native refusal: {events:?}"
    );
}

fn add<E: Equilibrator>(bench: &mut Bench, solver: &mut E, key: &str, amount: f64) {
    step(
        bench,
        solver,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(key),
            moles: Moles(amount),
            at: None,
        },
    );
}

#[test]
fn sealed_single_valence_ammonia_has_no_earned_pe_after_acid_trapping() {
    let mut bench = Bench::new();
    bench.vessels[0].thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
    let mut solver =
        kerotakis_stack::standard_stack(vec![Box::new(PhreeqcEquilibrator::new().unwrap())]);
    add(&mut bench, &mut solver, "water", 3.873_993_893_977_241_6);
    step(
        &mut bench,
        &mut solver,
        Operator::Seal {
            vessel: VesselId(0),
            headspace_volume: Liters(0.035),
        },
    );
    let inert_n2 = bench.vessels[0].moles_of(&SpeciesId::new("N2")).0;
    let inert_o2 = bench.vessels[0].moles_of(&SpeciesId::new("O2")).0;
    add(&mut bench, &mut solver, "HCl", 0.00016);
    add(&mut bench, &mut solver, "NH3", 0.00008);
    let vessel = &bench.vessels[0];
    let info = vessel.solution.as_ref().expect("complete native answer");
    assert_eq!(info.scope, vessel::SolutionScope::Complete);
    assert_eq!(info.pe, None, "single N(-3) does not determine pe");
    assert!(info
        .redox
        .iter()
        .any(|state| state.element == "N" && state.oxidation == -3));
    assert!(info.ph.is_finite() && info.ionic_strength.is_finite());
    assert!((vessel.moles_of(&SpeciesId::new("N2")).0 - inert_n2).abs() < 1e-12);
    assert!((vessel.moles_of(&SpeciesId::new("O2")).0 - inert_o2).abs() < 1e-12);
    assert!(vessel.moles_of(&SpeciesId::new("NO3-")).0 < 1e-12);
    // A zero-time typed repeat is a fresh native readback, not a display
    // snapshot, and must preserve the same withholding decision.
    step(&mut bench, &mut solver, Operator::Wait { seconds: 0.0 });
    assert_eq!(bench.vessels[0].solution.as_ref().unwrap().pe, None);
}

fn redox_half_titration(solver: &mut PhreeqcEquilibrator, permanganate: f64) -> Bench {
    let mut bench = Bench::new();
    add(&mut bench, solver, "water", 5.55);
    add(&mut bench, solver, "H2SO4", 0.005);
    add(&mut bench, solver, "FeSO4", 0.005);
    add(&mut bench, solver, "KMnO4", permanganate);
    bench
}

#[test]
fn actual_coupled_root_survives_new_cache_round_trip_but_endpoint_does_not_gain_pe() {
    let mut fresh = PhreeqcEquilibrator::new().expect("engine");
    let half = redox_half_titration(&mut fresh, 0.0005);
    assert!(half.vessels[0].solution.as_ref().unwrap().pe.is_some());
    let cache = fresh.export_cache();
    let cache_json = serde_json::to_value(&cache).unwrap();
    assert!(
        cache_json["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| { entry["pe_authority"] == "coupled_root" }),
        "a produced coupled answer must carry its root witness"
    );
    let mut imported = PhreeqcEquilibrator::new().expect("engine");
    imported.import_cache(serde_json::from_value(cache_json).unwrap());
    let mut replay = Bench::new();
    add(&mut replay, &mut imported, "water", 5.55);
    add(&mut replay, &mut imported, "H2SO4", 0.005);
    add(&mut replay, &mut imported, "FeSO4", 0.005);
    let hits_before_root = imported.cache_hits();
    let calls_before_root = imported.engine_calls();
    add(&mut replay, &mut imported, "KMnO4", 0.0005);
    assert!(
        imported.cache_hits() > hits_before_root,
        "the actual root entry was reused"
    );
    assert_eq!(
        imported.engine_calls(),
        calls_before_root,
        "the imported coupled-root answer must not be recomputed"
    );
    assert!(replay.vessels[0].solution.as_ref().unwrap().pe.is_some());

    let mut endpoint_solver = PhreeqcEquilibrator::new().expect("engine");
    let endpoint = redox_half_titration(&mut endpoint_solver, 0.0010);
    assert_eq!(endpoint.vessels[0].solution.as_ref().unwrap().pe, None);
}

#[test]
fn open_ferric_pe_remains_posed_and_native_mix_does_not_borrow_source_poises() {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let mut ferric = Bench::new();
    add(&mut ferric, &mut solver, "water", 5.551);
    add(&mut ferric, &mut solver, "FeCl3", 0.01);
    assert!(ferric.vessels[0]
        .solution
        .as_ref()
        .unwrap()
        .pe
        .is_some_and(|pe| pe > 10.0));

    let source = |id, ion: &str, amount: f64| {
        let mut vessel = Vessel::new(VesselId(id), "open mix pe owner");
        vessel.thermal_mode = ThermalMode::Thermostatted(Kelvin::STANDARD);
        vessel.deposit(SpeciesId::new("water"), Moles(2.775), Phase::Liquid);
        vessel.deposit(SpeciesId::new(ion), Moles(amount), Phase::Aqueous);
        vessel.deposit(SpeciesId::new("Cl-"), Moles(amount), Phase::Aqueous);
        vessel
    };
    let a = source(0, "Na+", 0.001);
    let b = source(1, "NH4+", 0.002);
    let mut target = Vessel::new(VesselId(2), "mixed represented stock");
    target.thermal_mode = ThermalMode::Thermostatted(Kelvin::STANDARD);
    for vessel in [&a, &b] {
        for portion in &vessel.contents {
            target.deposit(portion.species.clone(), portion.moles, portion.phase);
        }
    }
    solver
        .mix(&mut target, &a, 1.0, &b, 1.0)
        .expect("uncoupled native MIX remains available")
        .expect("native MIX answers");
    let info = target.solution.as_ref().expect("mixed native state");
    assert!(info.ph.is_finite() && !info.redox.is_empty());
    assert_eq!(
        info.pe, None,
        "the target MIX did not impose an electron boundary"
    );
}
