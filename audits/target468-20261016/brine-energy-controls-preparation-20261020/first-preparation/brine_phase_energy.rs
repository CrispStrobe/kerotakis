//! Frozen direct phase-energy controls. Preparation only: not executed.
//! Stage as crates/kerotakis-core/tests/brine_phase_energy.rs after prerequisites.
use kerotakis_core::{Equilibrator, Kelvin, Moles, Phase, SpeciesId, Vessel, VesselId};
use serde_json::Value;

fn water_record() -> Value {
    let rows: Vec<Value> = serde_json::from_str(include_str!("golden/registry.json")).unwrap();
    rows.into_iter().find(|row| row["key"] == "water").unwrap()
}
// Independent quadrature of authenticated registry coefficients. Never call
// the production energy/settling/root helpers as an energy oracle.
fn cp(row: &Value, phase: Phase, temperature: f64) -> f64 {
    let label = match phase {
        Phase::Solid => "solid",
        Phase::Liquid => "liquid",
        _ => panic!("only condensed water belongs in these controls"),
    };
    let records = row["heat_capacity_polys"].as_array().unwrap();
    let Some(record) = records.iter().find(|record| record["phase"] == label) else {
        return row["heat_capacity"].as_f64().unwrap();
    };
    let intervals = record["intervals"].as_array().unwrap();
    let t = temperature.clamp(
        intervals.first().unwrap()["t_min"].as_f64().unwrap(),
        intervals.last().unwrap()["t_max"].as_f64().unwrap(),
    );
    let interval = intervals
        .iter()
        .find(|interval| {
            interval["t_min"].as_f64().unwrap() <= t && t <= interval["t_max"].as_f64().unwrap()
        })
        .unwrap();
    assert_eq!(interval["form"], "nasa9");
    let a: Vec<f64> = interval["coefficients"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_f64().unwrap())
        .collect();
    8.314_462_618_153_24
        * (a[0] / t.powi(2)
            + a[1] / t
            + a[2]
            + a[3] * t
            + a[4] * t.powi(2)
            + a[5] * t.powi(3)
            + a[6] * t.powi(4))
}
fn sensible(row: &Value, phase: Phase, target: f64) -> f64 {
    const N: usize = 4096;
    let step = (target - 273.15) / N as f64;
    let mut sum = 0.0;
    for i in 0..=N {
        let weight = if i == 0 || i == N {
            1.0
        } else if i % 2 == 0 {
            2.0
        } else {
            4.0
        };
        sum += weight * cp(row, phase, 273.15 + i as f64 * step);
    }
    sum * step / 3.0
}
fn energy(vessel: &Vessel) -> f64 {
    let row = water_record();
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "water")
        .map(|p| {
            p.moles.0
                * (sensible(&row, p.phase, vessel.temperature.0)
                    - if p.phase == Phase::Solid { 6010.0 } else { 0.0 })
        })
        .sum::<f64>()
        + vessel.excess_enthalpy_j
}
fn ice(vessel: &Vessel) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "water" && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum()
}
fn check(salt_moles: f64, temperature: f64, freezing: bool) {
    let mut vessel = Vessel::new(VesselId(0), "direct declared phase-energy control");
    vessel.temperature = Kelvin(temperature);
    vessel.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Solid);
    vessel.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    if salt_moles > 0.0 {
        vessel.deposit(SpeciesId::new("Na+"), Moles(salt_moles), Phase::Aqueous);
        vessel.deposit(SpeciesId::new("Cl-"), Moles(salt_moles), Phase::Aqueous);
    }
    let before = vessel.clone();
    let initial_energy = energy(&before);
    let events = kerotakis_core::solve::StateEquilibrator
        .equilibrate(&mut vessel)
        .unwrap();
    assert!(
        !events.is_empty(),
        "control must exercise a real phase transition"
    );
    assert!(vessel.temperature.0.is_finite() && vessel.temperature.0 > 0.0);
    assert!(ice(&vessel) > 0.0 && ice(&vessel) < 2.0);
    assert!((vessel.moles_of(&SpeciesId::new("water")).0 - 2.0).abs() < 1e-12);
    for reagent in ["Na+", "Cl-"] {
        assert!((vessel.moles_of(&SpeciesId::new(reagent)).0 - salt_moles).abs() < 1e-12);
    }
    if freezing {
        assert!(ice(&vessel) > ice(&before) + 1e-12);
    } else {
        assert!(ice(&vessel) < ice(&before) - 1e-12);
    }
    let expected = if freezing {
        (Phase::Liquid, Phase::Solid)
    } else {
        (Phase::Solid, Phase::Liquid)
    };
    assert!(events.iter().any(|event| matches!(event,
        kerotakis_core::Event::StateChanged { species, from, to, .. }
            if species.0 == "water" && (*from, *to) == expected
    )));
    assert!(vessel
        .contents
        .iter()
        .all(|portion| portion.moles.0.is_finite() && portion.moles.0 >= 0.0));
    let residual = energy(&vessel) - initial_energy;
    assert!(residual.abs() <= 0.001, "unowned phase energy {residual} J");
}
#[test]
fn pure_water_freezing_conserves_independent_phase_energy() {
    check(0.0, 263.15, true);
}
#[test]
fn pure_water_melting_conserves_independent_phase_energy() {
    check(0.0, 273.65, false);
}
#[test]
fn dilute_brine_freezing_conserves_independent_phase_energy() {
    check(0.001, 263.15, true);
}
#[test]
fn intermediate_brine_freezing_conserves_independent_phase_energy() {
    check(0.01, 263.15, true);
}
#[test]
fn stronger_brine_freezing_conserves_independent_phase_energy() {
    check(0.03, 263.15, true);
}
#[test]
fn dilute_brine_melting_conserves_independent_phase_energy() {
    check(0.001, 273.05, false);
}
#[test]
fn intermediate_brine_melting_conserves_independent_phase_energy() {
    check(0.01, 273.05, false);
}
#[test]
fn stronger_brine_melting_conserves_independent_phase_energy() {
    check(0.03, 273.05, false);
}
