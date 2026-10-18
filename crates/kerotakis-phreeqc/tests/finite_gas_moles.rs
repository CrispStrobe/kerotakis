#![cfg(feature = "engine")]
//! Finite inventory is supplied in moles, independent of native gas EOS.
use kerotakis_core::{
    ledger::ConservedLedger, Equilibrator, Headspace, Kelvin, Liters, Moles, Phase, SpeciesId,
    Vessel, VesselId,
};
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn finite_carbon(salt: f64, gas: f64, temperature: f64) {
    let mut solver = PhreeqcEquilibrator::new().expect("real native engine");
    let mut vessel = Vessel::new(VesselId(0), "finite gas molar boundary");
    vessel.temperature = Kelvin(temperature);
    vessel.headspace = Headspace::Sealed {
        volume: Liters(0.4),
    };
    vessel.deposit(
        SpeciesId::new("water"),
        Moles(0.08 * 1000.0 / 18.015),
        Phase::Liquid,
    );
    vessel.deposit(SpeciesId::new("Na+"), Moles(salt), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("Cl-"), Moles(salt), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("CO2(aq)"), Moles(0.0001), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("CO2"), Moles(gas), Phase::Gas);
    let before = ConservedLedger::from_vessel(&vessel);
    for _ in 0..4 {
        solver
            .equilibrate(&mut vessel)
            .expect("finite native equilibrium");
        assert!(
            vessel.solution.is_some(),
            "solver did not return an aqueous state"
        );
        let after = ConservedLedger::from_vessel(&vessel);
        for element in ["C", "H", "O", "Na", "Cl"] {
            let expected = before.elements.get(element).copied().unwrap_or(0.0);
            let actual = after.elements.get(element).copied().unwrap_or(0.0);
            assert!(
                (actual - expected).abs() < 1e-10,
                "salt={salt} gas={gas} T={temperature} {element}: {expected} -> {actual}"
            );
        }
        assert!(
            (after.charge - before.charge).abs() < 1e-10,
            "charge drift: {} -> {}",
            before.charge,
            after.charge
        );
        assert!(
            vessel.moles_of(&SpeciesId::new("CO2")).0 > 0.0,
            "finite dissolved carbon must be able to form a headspace phase"
        );
    }
}

#[test]
fn ideal_and_real_gas_databases_preserve_finite_carbon_across_settlement() {
    for salt in [0.0, 0.08] {
        for gas in [0.001, 0.007] {
            for temperature in [298.15, 310.123456789] {
                finite_carbon(salt, gas, temperature);
            }
        }
    }
}

#[test]
fn initially_empty_finite_gas_phase_can_form_without_an_external_reservoir() {
    for salt in [0.0, 0.08] {
        finite_carbon(salt, 0.0, 298.15);
    }
}
