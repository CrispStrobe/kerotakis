//! Reference heat is per consumed H2O2, not per 2H2O2 reaction extent.
//! NIST TN2126, Table4-4(1), printed p328/PDF p337: liquid H2O2
//! reference release98.48 +/-0.79 kJ/mol at298.15K. This is a declared
//! reference-heat/stoichiometry check, not aqueous mixing/rate calibration.
use kerotakis_core::clock::{Clock, ClockContext, CuratedKineticsClock};
use kerotakis_core::vessel::ThermalMode;
use kerotakis_core::{Event, Kelvin, Moles, Phase, SpeciesId, Vessel, VesselId};
use serde_json::Value;

fn vessel(peroxide: f64, iodide: f64, mode: ThermalMode) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "peroxide heat basis");
    v.temperature = Kelvin(298.15);
    v.thermal_mode = mode;
    v.deposit(SpeciesId::new("water"), Moles(10.0), Phase::Liquid);
    v.deposit(SpeciesId::new("H2O2"), Moles(peroxide), Phase::Liquid);
    v.deposit(SpeciesId::new("KI"), Moles(iodide), Phase::Aqueous);
    v
}
fn context(exclude_peroxide: bool) -> ClockContext {
    ClockContext {
        excluded_kinetic_reactions: kerotakis_core::kinetics::REGISTRY
            .iter()
            .filter(|r| exclude_peroxide || r.id != "peroxide-decomposition")
            .map(|r| r.id.to_string())
            .collect(),
        ..Default::default()
    }
}
fn advance(v: &mut Vessel, seconds: f64, excluded: bool) -> Vec<Event> {
    let mut events = Vec::new();
    CuratedKineticsClock
        .advance(v, seconds, &context(excluded), &mut events)
        .unwrap();
    events
}
fn heat(events: &[Event]) -> f64 {
    events
        .iter()
        .filter_map(|e| match e {
            Event::ReactionHeatReleased {
                reaction, energy_j, ..
            } if reaction == "peroxide-decomposition" => Some(*energy_j),
            _ => None,
        })
        .sum()
}
fn check_conversion(before: &Vessel, after: &Vessel, events: &[Event]) -> f64 {
    let n = |v: &Vessel, key| v.moles_of(&SpeciesId::new(key)).0;
    let consumed = n(before, "H2O2") - n(after, "H2O2");
    assert!(consumed > 1e-8 && consumed < n(before, "H2O2"));
    assert!((n(after, "O2") - n(before, "O2") - consumed / 2.0).abs() < 1e-12);
    assert!((n(after, "water") - n(before, "water") - consumed).abs() < 1e-12);
    assert_eq!(n(before, "KI"), n(after, "KI"));
    let q = heat(events);
    assert!(q.is_finite() && q > 0.0);
    // Independent literature band; never derive the expected heat from the
    // production coefficient, emitted extent, or a number assigned by a mock.
    assert!(
        (q / consumed - 98_480.0).abs() <= 790.0,
        "reference heat basis mismatch: {q} J for {consumed} mol H2O2"
    );
    let oxygen_event: f64 = events
        .iter()
        .filter_map(|e| match e {
            Event::GasProduced {
                reaction,
                species,
                moles,
                ..
            } if reaction == "peroxide-decomposition" && species.0 == "O2" => Some(moles.0),
            _ => None,
        })
        .sum();
    assert!((oxygen_event - consumed / 2.0).abs() < 1e-12);
    q
}
fn cp(row: &Value, phase: Phase, t: f64) -> f64 {
    let phase = match phase {
        Phase::Liquid => "liquid",
        Phase::Solid => "solid",
        Phase::Gas => "gas",
        Phase::Aqueous => "liquid",
    };
    let Some(curve) = row["heat_capacity_polys"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["phase"] == phase)
    else {
        return row["heat_capacity"].as_f64().unwrap();
    };
    let rows = curve["intervals"].as_array().unwrap();
    let t = t.clamp(
        rows.first().unwrap()["t_min"].as_f64().unwrap(),
        rows.last().unwrap()["t_max"].as_f64().unwrap(),
    );
    let r = rows
        .iter()
        .find(|r| r["t_min"].as_f64().unwrap() <= t && t <= r["t_max"].as_f64().unwrap())
        .unwrap();
    assert_eq!(r["form"], "nasa9");
    let a: Vec<_> = r["coefficients"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
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
fn independent_sensible(v: &Vessel, from: f64) -> f64 {
    let rows: Vec<Value> = serde_json::from_str(include_str!("golden/registry.json")).unwrap();
    let step = (v.temperature.0 - from) / 4096.0;
    v.contents
        .iter()
        .map(|p| {
            let r = rows.iter().find(|r| r["key"] == p.species.0).unwrap();
            let integral: f64 = (0..=4096)
                .map(|i| {
                    let w = if i == 0 || i == 4096 {
                        1.0
                    } else if i % 2 == 0 {
                        2.0
                    } else {
                        4.0
                    };
                    w * cp(r, p.phase, from + i as f64 * step)
                })
                .sum();
            p.moles.0 * integral * step / 3.0
        })
        .sum()
}
#[test]
fn thermostatted_peroxide_heat_uses_consumed_molecules_not_extent() {
    let mut v = vessel(0.01, 0.001, ThermalMode::Thermostatted(Kelvin(298.15)));
    let before = v.clone();
    let events = advance(&mut v, 10.0, false);
    check_conversion(&before, &v, &events);
    assert_eq!(v.temperature, before.temperature);
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::TemperatureChanged { .. })));
}
#[test]
fn second_loading_preserves_the_reference_heat_basis() {
    let mut v = vessel(0.02, 0.0005, ThermalMode::Thermostatted(Kelvin(298.15)));
    let before = v.clone();
    let events = advance(&mut v, 3.0, false);
    check_conversion(&before, &v, &events);
    assert_eq!(v.temperature, before.temperature);
}
#[test]
fn adiabatic_peroxide_heat_matches_independent_sensible_energy() {
    let mut v = vessel(0.01, 0.001, ThermalMode::Adiabatic);
    let before = v.clone();
    let events = advance(&mut v, 5.0, false);
    let q = check_conversion(&before, &v, &events);
    assert!(v.temperature.0 > before.temperature.0);
    assert!((independent_sensible(&v, before.temperature.0) - q).abs() < 1e-5);
}
#[test]
fn excluded_peroxide_path_produces_no_material_or_heat() {
    let mut v = vessel(0.01, 0.001, ThermalMode::Adiabatic);
    let before = v.clone();
    let events = advance(&mut v, 10.0, true);
    assert_eq!(v.contents, before.contents);
    assert_eq!(v.temperature, before.temperature);
    assert!(events.is_empty());
}
