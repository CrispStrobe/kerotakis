//! Regressions discovered by independently predicted CLI experiments.
#![cfg(feature = "engine")]
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        DisplacementEquilibrator::wrapping(Box::new(PhreeqcEquilibrator::new().unwrap())),
    )]))
}
fn run(bench: &mut Bench, stack: &mut SolverStack, line: &str) -> Vec<Event> {
    bench
        .step_with(
            script::parse_op(line).unwrap().unwrap(),
            stack,
            &PermissiveScreen,
        )
        .unwrap()
}
fn aqueous_carbon(v: &Vessel) -> (f64, f64) {
    let inventory = v
        .contents
        .iter()
        .filter(|p| ["CO2(aq)", "HCO3-", "CO3-2"].contains(&p.species.0.as_str()))
        .map(|p| p.moles.0)
        .sum::<f64>();
    let info = v.solution.as_ref().unwrap();
    let speciation = info
        .species
        .iter()
        .filter(|p| ["CO2", "H2CO3", "HCO3-", "CO3-2"].contains(&p.name.as_str()))
        .map(|p| p.molality)
        .sum::<f64>()
        * info.solvent_kg.unwrap();
    (inventory, speciation)
}
#[test]
fn finite_carbon_dose_and_reported_speciation_agree_before_and_after_wait() {
    let mut b = Bench::new();
    let mut s = stack();
    run(&mut b, &mut s, "add v1 water 100mL");
    run(&mut b, &mut s, "add v1 CO2 0.001mol");
    let v = b.vessel(VesselId(0)).unwrap();
    let (inventory, speciation) = aqueous_carbon(v);
    assert!((inventory - 0.001).abs() < 1e-9);
    assert!(
        (inventory - speciation).abs() < 1e-9 + inventory.abs() * 2e-5,
        "inventory {inventory}, speciation {speciation}"
    );
    assert!(
        v.solution.as_ref().unwrap().ph < 4.5,
        "finite dose must not report atmospheric pH"
    );
    for line in ["wait 1s", "heat v1 1J"] {
        run(&mut b, &mut s, line);
        let (inventory, speciation) = aqueous_carbon(b.vessel(VesselId(0)).unwrap());
        assert!(
            (inventory - speciation).abs() < 1e-9 + inventory.abs() * 2e-5,
            "{line}: inventory {inventory}, speciation {speciation}"
        );
    }
}
#[test]
fn potassium_nitrate_absorbs_its_reviewed_dissolution_heat() {
    let mut b = Bench::new();
    let mut s = stack();
    run(&mut b, &mut s, "add v1 water 100mL");
    run(&mut b, &mut s, "add v1 KNO3 0.01mol");
    let v = b.vessel(VesselId(0)).unwrap();
    let cooling = 298.15 - v.temperature.0;
    // 0.01 mol * 34.9 kJ/mol / about 418 J/K = about 0.84 K.
    assert!((0.7..1.0).contains(&cooling), "cooling {cooling} K");
    assert!(v.unpriced_heat.is_empty(), "{v:?}");
}
#[test]
fn unpriced_hydrate_heat_does_not_silently_become_a_known_temperature() {
    let mut b = Bench::new();
    let mut s = stack();
    run(&mut b, &mut s, "add v1 water 100mL");
    let events = run(&mut b, &mut s, "add v1 chalcanthite 0.001mol");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::HeatUnpriced { .. })),
        "{events:?}"
    );
    assert!(b.vessel(VesselId(0)).unwrap().solution.is_some());
    let events = run(&mut b, &mut s, "measure v1 thermometer");
    assert!(
        events.iter().any(
            |e| matches!(e, Event::Measured { note_reason: Some(reason), .. }
        if reason.key == "measurement.temperature-incomplete")
        ),
        "{events:?}"
    );
}
#[test]
fn unsupported_isopropanol_ignition_preserves_fuel_and_does_not_commit_boiling() {
    let mut b = Bench::new();
    let mut s = stack();
    run(&mut b, &mut s, "add v1 isopropanol 10mL");
    let before = b.vessel(VesselId(0)).unwrap().contents.clone();
    let events = run(&mut b, &mut s, "ignite v1");
    assert_eq!(b.vessel(VesselId(0)).unwrap().contents, before);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::StateChanged { .. } | Event::GasEvolved { .. })),
        "{events:?}"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })),
        "missing combustion is explicit"
    );
}
#[test]
fn zinc_in_acid_reports_missing_rate_instead_of_computed_inertness() {
    let mut b = Bench::new();
    let mut s = stack();
    run(&mut b, &mut s, "add v1 water 100mL");
    run(&mut b, &mut s, "add v1 HCl 0.01mol");
    let events = run(&mut b, &mut s, "add v1 Zn 0.001mol");
    assert!(
        events.iter().any(
            |e| matches!(e, Event::NotYetModeled { reason: Some(reason), .. }
        if reason.key == "not-modeled.hydrogen-overpotential-rate")
        ),
        "{events:?}"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::Inert { species, computed: true, .. }
        if species.0 == "Zn")),
        "{events:?}"
    );
}
#[test]
fn gas_electrolysis_explanation_matches_the_hydrogen_product() {
    let mut b = Bench::new();
    let mut s = stack();
    run(&mut b, &mut s, "add v1 water 100mL");
    run(&mut b, &mut s, "add v1 Na2SO4 0.001mol");
    let events = run(&mut b, &mut s, "electrolyse v1 0.1A 100s");
    let text = render_events_in(&events, Register(3), Locale::EN).join("\n");
    assert!(text.contains("hydrogen"), "{text}");
    assert!(!text.contains("hydrogen is not co-evolved"), "{text}");
    assert!(
        !text.contains("weigh"),
        "gas electrolysis is not a plated-electrode mass: {text}"
    );
}
