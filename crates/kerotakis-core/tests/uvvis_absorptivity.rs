//! Cited single-wavelength UV-vis absorptivities (lane F, 2026-11-11).
//!
//! `uvvis_absorptivity_coverage_is_current` pins the runtime table to
//! docs/uvvis-absorptivity-coverage-20261111.json
//! (`python3 tools/uvvis_absorptivity_coverage.py` regenerates it); the rest
//! are positive and negative controls of the generic Beer–Lambert sum.

use kerotakis_core::iodine_chemistry as iodine;
use kerotakis_core::units::Kelvin;
use kerotakis_core::uvvis_absorptivity::{self as uv, NO_DATA, TABLE};
use kerotakis_core::vessel::{SolutionInfo, SpeciesDetail};
use kerotakis_core::*;

const COVERAGE: &str = include_str!("../../../docs/uvvis-absorptivity-coverage-20261111.json");

#[test]
fn uvvis_absorptivity_coverage_is_current() {
    let doc: serde_json::Value = serde_json::from_str(COVERAGE).unwrap();
    let rows = doc["rows"].as_array().unwrap();
    assert_eq!(rows.len(), TABLE.len());
    for (row, cited) in rows.iter().zip(TABLE) {
        assert_eq!(row["species"].as_str().unwrap(), cited.species);
        assert_eq!(row["nm"].as_f64().unwrap(), cited.nm);
        assert_eq!(row["epsilon"].as_f64().unwrap(), cited.epsilon);
        assert_eq!(row["confidence"].as_str().unwrap(), cited.confidence.key());
        let t: Option<Vec<f64>> = row["temperature"]
            .as_array()
            .map(|a| a.iter().map(|v| v.as_f64().unwrap()).collect());
        assert_eq!(t, cited.temperature.map(|(c, lo, hi)| vec![c, lo, hi]));
        assert!(!cited.source.is_empty());
    }
    let refused: Vec<&str> = doc["no_data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["species"].as_str().unwrap())
        .collect();
    let expected: Vec<&str> = NO_DATA.iter().map(|(k, _)| *k).collect();
    assert_eq!(refused, expected);
    // The iodine rows and the 16-band iodine constants say the same thing.
    let i2 = TABLE.iter().find(|r| r.species == "I2").unwrap();
    assert_eq!(i2.epsilon, iodine::I2_BAND.1);
}

fn water(ml: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(ml / 18.015), Phase::Liquid);
    v
}

fn aq(v: &mut Vessel, key: &str, n: f64) {
    v.deposit(SpeciesId::new(key), Moles(n), Phase::Aqueous);
}

fn native(v: &mut Vessel, species: &[(&str, f64)]) {
    let kg = v.moles_of(&SpeciesId::new("water")).0 * 0.018015;
    v.solution = Some(SolutionInfo {
        solvent_activity: None,
        scope: Default::default(),
        solvent_kg: Some(kg),
        particle_molality: None,
        ph: 2.0,
        pe: None,
        redox: vec![],
        ionic_strength: 0.01,
        provenance: None,
        species: species
            .iter()
            .map(|(name, molality)| SpeciesDetail {
                name: name.to_string(),
                molality: *molality,
                activity: *molality,
            })
            .collect(),
    });
}

fn measure(v: Vessel) -> Vec<Event> {
    let mut bench = Bench::new();
    bench.vessels[0] = v;
    bench
        .step_with(
            Operator::Measure {
                vessel: VesselId(0),
                instrument: Instrument::Spectrophotometer,
            },
            &mut SolverStack::new(vec![]),
            &PermissiveScreen,
        )
        .unwrap()
}

fn cited_measured(events: &[Event], nm: &str) -> Option<f64> {
    events.iter().find_map(|e| match e {
        Event::Measured { value, unit, .. }
            if unit == &format!("absorbance at {nm} nm (cited molar absorptivity)") =>
        {
            Some(*value)
        }
        _ => None,
    })
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * b.abs().max(1e-12)
}

#[test]
fn permanganate_reads_its_cited_epsilon_at_525_and_545() {
    let mut v = water(100.0);
    aq(&mut v, "MnO4-", 1e-5);
    let litres = v.liquid_volume().0;
    let r525 = uv::absorbance_at(&v, 525.0, 1.0);
    assert!(r525.blocked.is_empty(), "{:?}", r525.blocked);
    assert!(close(r525.absorbance, 3340.0 * 1e-5 / litres));
    let r545 = uv::absorbance_at(&v, 545.0, 2.0);
    assert!(close(r545.absorbance, 3220.0 * 1e-5 / litres * 2.0));
    let events = measure(v);
    let a = cited_measured(&events, "525").expect("cited 525 nm reading");
    assert!(close(a, 3340.0 * 1e-5 / litres));
    assert!(cited_measured(&events, "545").is_some());
}

#[test]
fn potassium_permanganate_carries_one_permanganate() {
    let mut a = water(100.0);
    aq(&mut a, "KMnO4", 2e-5);
    let mut b = water(100.0);
    aq(&mut b, "MnO4-", 2e-5);
    let ra = uv::absorbance_at(&a, 525.0, 1.0);
    assert!(ra.blocked.is_empty(), "{:?}", ra.blocked);
    assert!(close(
        ra.absorbance,
        uv::absorbance_at(&b, 525.0, 1.0).absorbance
    ));
}

#[test]
fn manganate_reads_at_606_while_the_band_reading_still_refuses() {
    let mut v = water(100.0);
    aq(&mut v, "MnO4-2", 1e-4);
    let litres = v.liquid_volume().0;
    let events = measure(v);
    let a = cited_measured(&events, "606").expect("cited 606 nm reading");
    assert!(close(a, 1710.0 * 1e-4 / litres));
    // The 16-band peak reading has no MnO4^2- band and keeps its refusal.
    assert!(events.iter().any(|e| matches!(e,
        Event::NotYetModeled { reason: Some(r), .. }
        if r.key == "not-modeled.unreviewed-molar-absorptivity")));
}

#[test]
fn iodine_460_is_the_triiodide_split_with_temperature_coefficients() {
    let mut v = water(100.0);
    aq(&mut v, "I2", 1e-5);
    aq(&mut v, "KI", 1e-3);
    v.temperature = Kelvin(303.15);
    let litres = v.liquid_volume().0;
    let s = iodine::optical_split(&v).unwrap();
    let expected = (722.0 * (1.0 - 0.0019 * 5.0) * s.iodine
        + 1022.0 * (1.0 + 0.0048 * 5.0) * s.triiodide)
        / litres;
    let r = uv::absorbance_at(&v, 460.0, 1.0);
    assert!(r.blocked.is_empty(), "{:?}", r.blocked);
    assert!(
        close(r.absorbance, expected),
        "{} vs {expected}",
        r.absorbance
    );
}

#[test]
fn iodine_outside_its_measured_temperature_range_is_refused() {
    let mut v = water(100.0);
    aq(&mut v, "I2", 1e-5);
    v.temperature = Kelvin(323.15);
    let r = uv::absorbance_at(&v, 460.0, 1.0);
    assert!(!r.blocked.is_empty());
    let events = measure(v);
    assert!(cited_measured(&events, "460").is_none());
}

#[test]
fn dichromate_per_dichromate_equals_per_chromium_from_native_speciation() {
    // 1e-4 mol/kg Cr as HCrO4- plus 1e-5 mol/kg Cr2O7 2-: 1568 per Cr both ways.
    let mut v = water(100.0);
    native(
        &mut v,
        &[("HCrO4-", 1e-4), ("Cr2O7-2", 1e-5), ("K+", 1.2e-4)],
    );
    let kg = v.solution.as_ref().unwrap().solvent_kg.unwrap();
    let litres = v.liquid_volume().0;
    let r = uv::absorbance_at(&v, 350.0, 1.0);
    assert!(r.blocked.is_empty(), "{:?}", r.blocked);
    let chromium = (1e-4 + 2.0 * 1e-5) * kg / litres;
    assert!(close(r.absorbance, 1568.0 * chromium), "{}", r.absorbance);
}

#[test]
fn a_present_absorber_without_a_row_at_the_wavelength_refuses_typed() {
    let mut v = water(100.0);
    aq(&mut v, "MnO4-", 1e-5);
    aq(&mut v, "I2", 1e-5);
    assert_eq!(uv::absorbance_at(&v, 460.0, 1.0).blocked, vec!["MnO4-"]);
    assert_eq!(uv::absorbance_at(&v, 525.0, 1.0).blocked, vec!["I2"]);
    let events = measure(v);
    assert!(cited_measured(&events, "460").is_none());
    assert!(cited_measured(&events, "525").is_none());
    let reason = events
        .iter()
        .find_map(|e| match e {
            Event::NotYetModeled {
                reason: Some(r), ..
            } if r.key == "not-modeled.uvvis-absorptivity-no-data" => Some(r.clone()),
            _ => None,
        })
        .expect("typed refusal");
    for locale in [Locale::parse("de"), Locale::parse("fr")] {
        assert_ne!(reason.render(locale), reason.render(Locale::EN));
    }
}

#[test]
fn a_no_data_species_blocks_the_reading() {
    let mut v = water(100.0);
    native(&mut v, &[("Ni+2", 1e-2), ("Co+2", 1e-3), ("SO4-2", 1.1e-2)]);
    let r = uv::absorbance_at(&v, 395.0, 1.0);
    assert!(r.blocked.contains(&"Co+2".to_string()), "{:?}", r.blocked);
    // Without cobalt, nickel's aqua ion is read.
    let mut w = water(100.0);
    native(&mut w, &[("Ni+2", 1e-2), ("SO4-2", 1e-2)]);
    let r = uv::absorbance_at(&w, 395.0, 1.0);
    assert!(r.blocked.is_empty(), "{:?}", r.blocked);
    assert!(r.absorbance > 0.0);
}

#[test]
fn bromine_with_bromide_but_no_tribromide_split_is_refused() {
    let mut v = water(100.0);
    aq(&mut v, "Br2", 1e-4);
    let litres = v.liquid_volume().0;
    let r = uv::absorbance_at(&v, 390.0, 1.0);
    assert!(r.blocked.is_empty(), "{:?}", r.blocked);
    assert!(close(r.absorbance, 171.0 * 1e-4 / litres));
    aq(&mut v, "Br-", 1e-3);
    assert!(uv::absorbance_at(&v, 390.0, 1.0)
        .blocked
        .contains(&"Br3-".to_string()));
}

#[test]
fn starch_iodine_complex_still_refuses_the_cited_reading() {
    let mut v = water(100.0);
    aq(&mut v, "KI", 2.5e-4);
    aq(&mut v, "I2", 8e-5);
    v.deposit(SpeciesId::new("starch"), Moles(6e-3), Phase::Solid);
    let events = measure(v);
    assert!(!events.iter().any(|e| matches!(e, Event::Measured { .. })));
}

#[test]
fn a_vessel_without_cited_species_gets_no_cited_reading() {
    let mut v = water(100.0);
    aq(&mut v, "Na+", 1e-3);
    aq(&mut v, "Cl-", 1e-3);
    assert!(uv::readings(&v, 1.0).is_empty());
    let events = measure(v);
    assert!(!events.iter().any(|e| matches!(e,
        Event::Measured { unit, .. } if unit.contains("cited molar absorptivity"))));
}

#[test]
fn permanganate_band_peak_and_cited_525_reading_agree() {
    use kerotakis_core::instrument::{InstrumentContract, Spectrophotometer};
    for amount in [1e-6, 1e-5, 1e-4] {
        let mut v = water(100.0);
        aq(&mut v, "MnO4-", amount);
        let peak = Spectrophotometer::default()
            .measure(&v)
            .expect("band reading");
        assert_eq!(peak.observable, "absorbance at 525 nm");
        let cited = uv::absorbance_at(&v, 525.0, 1.0);
        assert!(cited.blocked.is_empty(), "{:?}", cited.blocked);
        assert!(
            (peak.value - cited.absorbance).abs() <= 1e-3 * cited.absorbance,
            "{} vs {}",
            peak.value,
            cited.absorbance
        );
    }
}

#[test]
fn alkaline_chromate_reads_haupts_nbs_absorptivity() {
    // Haupt 1952 (NBS RP2331) Table 1: 4815 L/mol/cm at 375 nm, 0.05 N KOH.
    let mut v = water(100.0);
    native(&mut v, &[("CrO4-2", 2e-4), ("K+", 4e-4)]);
    let kg = v.solution.as_ref().unwrap().solvent_kg.unwrap();
    let litres = v.liquid_volume().0;
    let r = uv::absorbance_at(&v, 375.0, 1.0);
    assert!(r.blocked.is_empty(), "{:?}", r.blocked);
    assert!(
        close(r.absorbance, 4815.0 * 2e-4 * kg / litres),
        "{}",
        r.absorbance
    );
}

#[test]
fn aqueous_cobalt_reads_the_nbs_filter_solution_absorptivity() {
    // Davis & Gibson 1931 (NBS MP114) Table 6: epsilon 4.75 at 510 nm.
    let mut v = water(100.0);
    native(&mut v, &[("Co+2", 0.05), ("SO4-2", 0.05)]);
    let kg = v.solution.as_ref().unwrap().solvent_kg.unwrap();
    let litres = v.liquid_volume().0;
    let r = uv::absorbance_at(&v, 510.0, 1.0);
    assert!(r.blocked.is_empty(), "{:?}", r.blocked);
    assert!(
        close(r.absorbance, 4.75 * 0.05 * kg / litres),
        "{}",
        r.absorbance
    );
}

#[test]
fn aqueous_chlorine_reads_fergussons_band_maximum() {
    // Fergusson, Slotin & Style 1936: epsilon about 66 at 334 nm.
    let mut v = water(100.0);
    native(&mut v, &[("Cl2", 1e-3)]);
    let kg = v.solution.as_ref().unwrap().solvent_kg.unwrap();
    let litres = v.liquid_volume().0;
    let r = uv::absorbance_at(&v, 334.0, 1.0);
    assert!(r.blocked.is_empty(), "{:?}", r.blocked);
    assert!(
        close(r.absorbance, 66.0 * 1e-3 * kg / litres),
        "{}",
        r.absorbance
    );
}
