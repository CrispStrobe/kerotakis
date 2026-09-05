//! BRD-041: below its autoignition temperature and without a spark, a fuel
//! does not burn — and the thermal solver says so instead of equilibrating.

use kerotakis_cea::ThermalEquilibrator;
use kerotakis_core::species::Phase;
use kerotakis_core::units::{Kelvin, Liters, Moles};
use kerotakis_core::vessel::{Headspace, Vessel, VesselId};
use kerotakis_core::{Equilibrator, Event, SpeciesId};

fn sealed(temperature_k: f64, feeds: &[(&str, f64, Phase)]) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "flask");
    v.headspace = Headspace::Sealed {
        volume: Liters(1.0),
    };
    v.temperature = Kelvin(temperature_k);
    for (key, moles, phase) in feeds {
        v.deposit(SpeciesId::new(key), Moles(*moles), *phase);
    }
    v.refresh_pressure();
    v
}

fn moles(v: &Vessel, key: &str) -> f64 {
    v.moles_of(&SpeciesId::new(key)).0
}

fn below(events: &[Event], fuel: &str) -> Option<(f64, f64)> {
    events.iter().find_map(|e| match e {
        Event::BelowAutoignition {
            fuel: f,
            autoignition,
            temperature,
            ..
        } if f.0 == fuel => Some((autoignition.0, temperature.0)),
        _ => None,
    })
}

#[test]
fn warm_methane_and_oxygen_do_not_burn_without_a_spark() {
    let mut v = sealed(
        700.0,
        &[("methane", 0.01, Phase::Gas), ("O2", 0.03, Phase::Gas)],
    );
    let mut cea = ThermalEquilibrator;
    assert!(
        cea.applies(&v),
        "above the 500 K threshold, CEA is consulted"
    );
    let events = cea.equilibrate(&mut v).expect("equilibrates");
    let (needs, at) = below(&events, "methane").expect("methane is named as below autoignition");
    assert!((needs - 810.15).abs() < 1e-9 && (at - 700.0).abs() < 1e-9);
    assert!(
        (moles(&v, "methane") - 0.01).abs() < 1e-12,
        "the fuel is untouched"
    );
    assert!(moles(&v, "CO2") < 1e-12, "nothing burned");
    assert!((v.temperature.0 - 700.0).abs() < 1e-9);
}

#[test]
fn above_autoignition_the_same_mixture_burns() {
    let mut v = sealed(
        900.0,
        &[("methane", 0.01, Phase::Gas), ("O2", 0.03, Phase::Gas)],
    );
    let mut cea = ThermalEquilibrator;
    let events = cea.equilibrate(&mut v).expect("equilibrates");
    assert!(below(&events, "methane").is_none(), "{events:?}");
    assert!(
        moles(&v, "methane") < 1e-6,
        "methane burned: {} left",
        moles(&v, "methane")
    );
    assert!(
        moles(&v, "CO2") > 0.005,
        "carbon dioxide formed: {}",
        moles(&v, "CO2")
    );
    assert!(v.temperature.0 > 900.0, "the flame heated the vessel");
}

#[test]
fn a_spark_is_above_every_autoignition_temperature() {
    // `ignite` takes a vessel to IGNITION_K; nothing in the table is hotter.
    for row in kerotakis_core::combustion::GAS_AUTOIGNITION {
        assert!(
            row.autoignition_k < kerotakis_core::IGNITION_K,
            "{}",
            row.species
        );
        assert!(!row.provenance.is_empty());
    }
}

#[test]
fn hydrogen_is_gated_too_and_without_oxygen_nothing_is_said() {
    let mut v = sealed(700.0, &[("H2", 0.02, Phase::Gas), ("O2", 0.01, Phase::Gas)]);
    let mut cea = ThermalEquilibrator;
    let events = cea.equilibrate(&mut v).expect("equilibrates");
    assert!(below(&events, "H2").is_some());
    assert!((moles(&v, "H2") - 0.02).abs() < 1e-12);

    // Fuel alone in a sealed flask: no oxygen, so no fire and no warning.
    let mut alone = sealed(700.0, &[("methane", 0.01, Phase::Gas)]);
    let events = cea.equilibrate(&mut alone).unwrap_or_default();
    assert!(below(&events, "methane").is_none(), "{events:?}");
}

#[test]
fn an_open_flask_of_warm_ethanol_is_below_autoignition_on_the_rooms_air() {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.temperature = Kelvin(550.0);
    v.deposit(SpeciesId::new("ethanol"), Moles(0.05), Phase::Liquid);
    let mut cea = ThermalEquilibrator;
    let events = cea.equilibrate(&mut v).expect("equilibrates");
    let (needs, _) = below(&events, "ethanol").expect("ethanol named");
    assert!((needs - 636.15).abs() < 1e-9);
    assert!((moles(&v, "ethanol") - 0.05).abs() < 1e-12);
}
