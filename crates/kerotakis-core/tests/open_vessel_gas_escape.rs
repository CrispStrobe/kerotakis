//! Lane E (2026-11-11): open-vessel gas escape through one two-film law.

use kerotakis_core::clock::{self, ClockContext};
use kerotakis_core::gas_escape::{self, EscapeGap};
use kerotakis_core::ops::{Event, NotModelledCause};
use kerotakis_core::{Moles, Phase, SpeciesId, Vessel, VesselId};

fn open_water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(11.1), Phase::Liquid);
    v
}

#[test]
fn covered_gases_scale_with_diffusivity_to_the_two_thirds() {
    for gas in ["CO2", "O2", "N2", "H2"] {
        assert!(
            gas_escape::transfer_velocity_m_per_s(gas, 298.15).is_ok(),
            "{gas} should be covered"
        );
    }
    let co2 = gas_escape::transfer_velocity_m_per_s("CO2", 298.15).unwrap();
    let h2 = gas_escape::transfer_velocity_m_per_s("H2", 298.15).unwrap();
    let expected = co2 * (4.50f64 / 1.92).powf(2.0 / 3.0);
    assert!(
        (h2 - expected).abs() < expected * 1e-12,
        "{h2} vs {expected}"
    );
    assert!(h2 > co2, "a faster diffuser leaves faster");
}

#[test]
fn gases_without_sourced_transfer_data_are_refused() {
    assert_eq!(
        gas_escape::transfer_velocity_m_per_s("NH3", 298.15),
        Err(EscapeGap::GasFilmUnsourced)
    );
    assert_eq!(
        gas_escape::transfer_velocity_m_per_s("Cl2", 298.15),
        Err(EscapeGap::GasFilmUnsourced)
    );
    assert_eq!(
        gas_escape::transfer_velocity_m_per_s("H2S", 298.15),
        Err(EscapeGap::NoHenryConstant)
    );
    assert_eq!(
        gas_escape::transfer_velocity_m_per_s("Br2", 298.15),
        Err(EscapeGap::NoHenryConstant)
    );
    assert_eq!(
        gas_escape::transfer_velocity_m_per_s("SO2", 298.15),
        Err(EscapeGap::NoDiffusivity)
    );
    assert_eq!(
        gas_escape::transfer_velocity_m_per_s("Xe", 298.15),
        Err(EscapeGap::NoDiffusivity)
    );
}

#[test]
fn escape_never_exceeds_the_excess() {
    assert_eq!(gas_escape::escaped_moles(1e-3, 1e-4, 0.0), 0.0);
    assert_eq!(gas_escape::escaped_moles(-1e-3, 1e-4, 100.0), 0.0);
    let long = gas_escape::escaped_moles(1e-3, 1e-4, 1e9);
    assert!(long <= 1e-3 && long > 0.999e-3);
}

#[test]
fn a_wait_on_open_dissolved_ammonia_is_refused_typed() {
    let mut v = open_water();
    v.deposit(SpeciesId::new("NH3"), Moles(0.01), Phase::Aqueous);
    let mut events = Vec::new();
    clock::advance(&mut v, 60.0, ClockContext::default(), &mut events).expect("advances");
    let refused: Vec<_> = events
        .iter()
        .filter(|e| {
            matches!(e, Event::NotYetModeled {
                cause: NotModelledCause::NotParameterised,
                reason: Some(reason), ..
            } if reason.key == "not-modeled.gas-escape-no-data")
        })
        .collect();
    assert_eq!(refused.len(), 1, "{events:?}");
}

#[test]
fn a_covered_gas_or_a_sealed_vessel_is_not_refused() {
    let mut v = open_water();
    v.deposit(SpeciesId::new("O2"), Moles(0.001), Phase::Aqueous);
    assert!(gas_escape::refusals(VesselId(0), &v).is_empty());

    let mut sealed = open_water();
    sealed.deposit(SpeciesId::new("NH3"), Moles(0.01), Phase::Aqueous);
    sealed.headspace = kerotakis_core::vessel::Headspace::Sealed {
        volume: kerotakis_core::units::Liters(0.05),
    };
    let mut events = Vec::new();
    clock::advance(&mut sealed, 60.0, ClockContext::default(), &mut events).expect("advances");
    assert!(!events.iter().any(|e| matches!(e, Event::NotYetModeled {
        reason: Some(reason), ..
    } if reason.key == "not-modeled.gas-escape-no-data")));
}

#[test]
fn supersaturated_carbon_dioxide_seeps_out_on_a_wait() {
    let mut v = open_water();
    v.temperature = kerotakis_core::units::Kelvin(298.15);
    v.co2_partial_pressure_atm = Some(0.05);
    let mut events = Vec::new();
    clock::advance(&mut v, 3600.0, ClockContext::default(), &mut events).expect("advances");
    assert!(
        v.pending_co2_transfer_mol < 0.0,
        "an hour open must move carbon out, got {}",
        v.pending_co2_transfer_mol
    );
}
