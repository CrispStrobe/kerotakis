//! Finite reactive gas doses use equilibrium, not experiment-specific yields.
#![cfg(feature = "engine")]
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

#[test]
fn reactive_gas_acid_conserves_finite_doses_and_neutralises_generically() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    for litres in [0.01, 0.1, 1.0] {
        for concentration in [1e-4, 1e-3, 1e-2] {
            for phase in [Phase::Gas, Phase::Aqueous] {
                let mut vessel = Vessel::new(VesselId(0), "beaker");
                vessel.thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
                vessel.deposit(
                    SpeciesId::new("water"),
                    species::lookup_key("water")
                        .unwrap()
                        .moles_from_liters(Liters(litres)),
                    Phase::Liquid,
                );
                let dose = litres * concentration;
                vessel.deposit(SpeciesId::new("HBr"), Moles(dose), phase);
                solver.equilibrate(&mut vessel).unwrap();
                let ph = vessel.solution.as_ref().unwrap().ph;
                assert!(
                    (ph + concentration.log10()).abs() < 0.1,
                    "{litres}, {concentration}, {phase:?}: {ph}"
                );
                let bromide: f64 = vessel
                    .contents
                    .iter()
                    .filter(|p| p.species.0 == "Br-")
                    .map(|p| p.moles.0)
                    .sum();
                assert!((bromide - dose).abs() < 1e-10 + dose * 1e-6);
                vessel.deposit(SpeciesId::new("KOH"), Moles(dose), Phase::Aqueous);
                solver.equilibrate(&mut vessel).unwrap();
                assert!(
                    (vessel.solution.as_ref().unwrap().ph - 7.0).abs() < 0.1,
                    "{vessel:?}"
                );
                solver.equilibrate(&mut vessel).unwrap();
                let bromide: f64 = vessel
                    .contents
                    .iter()
                    .filter(|p| p.species.0 == "Br-")
                    .map(|p| p.moles.0)
                    .sum();
                assert!(
                    (bromide - dose).abs() < 1e-10 + dose * 1e-6,
                    "no atmospheric HBr reservoir"
                );
            }
        }
    }
}

#[test]
fn sealed_reactive_gas_keeps_atoms_and_reaches_a_finite_headspace_equilibrium() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    for volume in [0.01, 0.1, 1.0] {
        let mut vessel = Vessel::new(VesselId(0), "beaker");
        vessel.thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
        vessel.headspace = Headspace::Sealed {
            volume: Liters(volume),
        };
        vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
        vessel.deposit(SpeciesId::new("HBr"), Moles(1e-3), Phase::Gas);
        let before = ledger::ConservedLedger::from_vessel(&vessel);
        solver.equilibrate(&mut vessel).unwrap();
        let after = ledger::ConservedLedger::from_vessel(&vessel);
        for (element, expected) in before.elements {
            assert!(
                (after.elements[&element] - expected).abs() < 1e-9 + expected * 1e-6,
                "{element}: expected {expected}, actual {}; {vessel:?}",
                after.elements[&element]
            );
        }
        assert!((vessel.solution.as_ref().unwrap().ph - 2.0).abs() < 0.15);
    }
}
