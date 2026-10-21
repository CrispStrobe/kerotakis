//! Physical phase resolution is independent of user-facing detection limits.
use kerotakis_core::{
    Equilibrator, Event, Kelvin, Moles, Phase, SpeciesId, StateEquilibrator, Vessel, VesselId,
};
use serde_json::Value;

fn independent_cp(phase: &str) -> f64 {
    let rows: Vec<Value> = serde_json::from_str(include_str!("golden/registry.json")).unwrap();
    let row = rows.iter().find(|row| row["key"] == "water").unwrap();
    let poly = row["heat_capacity_polys"]
        .as_array()
        .unwrap()
        .iter()
        .find(|poly| poly["phase"] == phase)
        .unwrap();
    let intervals = poly["intervals"].as_array().unwrap();
    let t = 273.15_f64.clamp(
        intervals.first().unwrap()["t_min"].as_f64().unwrap(),
        intervals.last().unwrap()["t_max"].as_f64().unwrap(),
    );
    let interval = intervals
        .iter()
        .find(|q| q["t_min"].as_f64().unwrap() <= t && t <= q["t_max"].as_f64().unwrap())
        .unwrap();
    assert_eq!(interval["form"], "nasa9");
    let c: Vec<f64> = interval["coefficients"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    8.314_462_618_153_24
        * (c[0] / t.powi(2)
            + c[1] / t
            + c[2]
            + c[3] * t
            + c[4] * t.powi(2)
            + c[5] * t.powi(3)
            + c[6] * t.powi(4))
}

fn check(sign: f64) {
    let mut vessel = Vessel::new(VesselId(0), "subobservable coexistence");
    vessel.temperature = Kelvin(273.15 + sign * 1e-6);
    vessel.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    vessel.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Solid);
    let expected = (independent_cp("liquid") + independent_cp("solid"))
        * (vessel.temperature.0 - 273.15).abs()
        / 6010.0;
    assert!(expected > 1e-12 && expected < kerotakis_core::OBSERVABLE_MOLES);
    let events = StateEquilibrator.equilibrate(&mut vessel).unwrap();
    let liquid = vessel
        .contents
        .iter()
        .find(|p| p.phase == Phase::Liquid)
        .unwrap()
        .moles
        .0;
    let solid = vessel
        .contents
        .iter()
        .find(|p| p.phase == Phase::Solid)
        .unwrap()
        .moles
        .0;
    assert!(((liquid - 1.0) - sign * expected).abs() < 1e-12);
    assert!((liquid + solid - 2.0).abs() < 1e-12);
    assert!((vessel.temperature.0 - 273.15).abs() < 1e-12);
    assert!(events.iter().any(|event| matches!(event, Event::StateChanged { moles, .. } if moles.0 > 0.0 && moles.0 < kerotakis_core::OBSERVABLE_MOLES)));
    let before = vessel.clone();
    let repeat = StateEquilibrator.equilibrate(&mut vessel).unwrap();
    assert!(!repeat
        .iter()
        .any(|event| matches!(event, Event::StateChanged { .. })));
    assert_eq!(vessel.temperature, before.temperature);
    assert_eq!(vessel.contents, before.contents);
}

#[test]
fn subobservable_freezing_is_a_real_conserved_phase_transfer() {
    check(-1.0);
}

#[test]
fn subobservable_melting_is_a_real_conserved_phase_transfer() {
    check(1.0);
}
