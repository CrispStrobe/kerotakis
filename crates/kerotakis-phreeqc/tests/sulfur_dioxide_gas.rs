//! SO2(g) over acidified sulfite: the Goldberg-Parker (1985) slice on the
//! minteq.v4 `Sulfite` pseudo-element (`aqueous_gases::sulfur_dioxide`).
//!
//! Expected numbers are computed from the slice's own constants, not
//! harvested. 0.01 mol Na2SO3 + 0.03 mol HCl in 0.1 kg water: 0.01 mol
//! strong acid is left after full protonation to H2SO3, so the S(IV)
//! balance is 0.01 = m0*0.1*(1 + K_A/[H+]) + (m0/K_D)*V/(RT) with
//! K_D = 10^0.09506 = 1.2447 mol kg-1 atm-1, K_A = 10^-1.85704, V = 0.5 L.
//! Ideal activities: [H+] 0.110 m, m0 0.0775 m, pSO2 = m0/K_D = 0.0622 atm,
//! n_gas 1.27e-3 mol. With Davies activities (I ~ 0.32): pH 1.07,
//! pSO2 0.0579 atm, n_gas 1.18e-3 mol. Both lie inside the +-10 % band
//! around the ideal values asserted below.
#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

const R_LITRE_ATM: f64 = 0.082_059_7;
const T: f64 = 298.15;

fn vessel(headspace: Headspace) -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.thermal_mode = ThermalMode::Thermostatted(Kelvin(T));
    vessel.headspace = headspace;
    vessel.deposit(
        SpeciesId::new("water"),
        species::lookup_key("water")
            .unwrap()
            .moles_from_liters(Liters(0.1)),
        Phase::Liquid,
    );
    vessel
}

fn amount(vessel: &Vessel, key: &str, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn s_iv(vessel: &Vessel) -> f64 {
    amount(vessel, "SO2", Phase::Aqueous)
        + amount(vessel, "SO2", Phase::Gas)
        + amount(vessel, "HSO3-", Phase::Aqueous)
        + amount(vessel, "SO3-2", Phase::Aqueous)
}

fn assert_conserved(before: &ledger::ConservedLedger, vessel: &Vessel) {
    let after = ledger::ConservedLedger::from_vessel(vessel);
    for (element, expected) in &before.elements {
        let actual = after.elements.get(element).copied().unwrap_or(0.0);
        assert!(
            (actual - expected).abs() < 1e-9 + expected.abs() * 1e-6,
            "{element}: expected {expected}, actual {actual}; {vessel:?}"
        );
    }
}

#[test]
fn sealed_acidified_sulfite_sets_the_goldberg_parker_so2_pressure() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    let mut vessel = vessel(Headspace::Sealed {
        volume: Liters(0.5),
    });
    vessel.deposit(SpeciesId::new("Na2SO3"), Moles(0.01), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("HCl"), Moles(0.03), Phase::Aqueous);
    let before = ledger::ConservedLedger::from_vessel(&vessel);
    solver.equilibrate(&mut vessel).unwrap();
    assert_conserved(&before, &vessel);

    let gas = amount(&vessel, "SO2", Phase::Gas);
    let pressure = gas * R_LITRE_ATM * T / 0.5;
    assert!(
        (pressure - 0.0622).abs() < 0.1 * 0.0622,
        "pSO2 {pressure} atm; {vessel:?}"
    );
    assert!((gas - 1.27e-3).abs() < 0.1 * 1.27e-3, "gas {gas} mol");
    // The neutral fraction is booked, never dropped: S(IV) closes exactly.
    assert!(
        (s_iv(&vessel) - 0.01).abs() < 1e-9,
        "S(IV) {}",
        s_iv(&vessel)
    );
    let neutral = amount(&vessel, "SO2", Phase::Aqueous);
    assert!(neutral > 0.006 && neutral < 0.0085, "SO2(aq) {neutral}");
    assert_eq!(amount(&vessel, "SO4-2", Phase::Aqueous), 0.0);
    let ph = vessel.solution.as_ref().unwrap().ph;
    assert!(ph > 0.85 && ph < 1.2, "pH {ph}");

    // A second solve is a fixed point, not a further transfer.
    solver.equilibrate(&mut vessel).unwrap();
    assert_conserved(&before, &vessel);
    let again = amount(&vessel, "SO2", Phase::Gas);
    assert!((again - gas).abs() < 1e-9 + gas * 1e-6, "{gas} -> {again}");
}

#[test]
fn supplied_so2_gas_dissolves_into_a_sealed_neutral_sulfite_solution() {
    // Finite SO2 over sodium sulfite: the gas is posed, books the Sulfite
    // element and is largely taken up as HSO3- (SO2 + SO3-2 + H2O = 2 HSO3-).
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    let mut vessel = vessel(Headspace::Sealed {
        volume: Liters(0.5),
    });
    vessel.deposit(SpeciesId::new("Na2SO3"), Moles(0.01), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("SO2"), Moles(0.005), Phase::Gas);
    let before = ledger::ConservedLedger::from_vessel(&vessel);
    solver.equilibrate(&mut vessel).unwrap();
    assert_conserved(&before, &vessel);
    assert!((s_iv(&vessel) - 0.015).abs() < 1e-9);
    assert!(amount(&vessel, "SO2", Phase::Gas) < 1e-4);
    assert!(amount(&vessel, "HSO3-", Phase::Aqueous) > 0.009);
    assert_eq!(amount(&vessel, "SO4-2", Phase::Aqueous), 0.0);
}

#[test]
fn sealed_sulfate_and_thiosulfate_gain_no_sulfur_dioxide() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    for (salt, ion, acid) in [("Na2SO4", "SO4-2", 0.03), ("Na2S2O3", "S2O3-2", 0.0)] {
        let mut vessel = vessel(Headspace::Sealed {
            volume: Liters(0.5),
        });
        vessel.deposit(SpeciesId::new(salt), Moles(0.01), Phase::Aqueous);
        if acid > 0.0 {
            vessel.deposit(SpeciesId::new("HCl"), Moles(acid), Phase::Aqueous);
        }
        let before = ledger::ConservedLedger::from_vessel(&vessel);
        solver.equilibrate(&mut vessel).unwrap();
        assert_conserved(&before, &vessel);
        assert_eq!(s_iv(&vessel), 0.0, "{salt}: {vessel:?}");
        if ion == "S2O3-2" {
            let left = amount(&vessel, ion, Phase::Aqueous);
            assert!((left - 0.01).abs() < 1e-9, "{salt}: {left}");
        }
    }
}

#[test]
fn an_open_beaker_loses_no_sulfur_dioxide_instantly() {
    // No ATMOSPHERIC SO2 entry and no open-vessel dose: acidified sulfite
    // in an open beaker keeps all of its S(IV) through an equilibrium solve.
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    let mut vessel = vessel(Headspace::Open);
    vessel.deposit(SpeciesId::new("Na2SO3"), Moles(0.01), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("HCl"), Moles(0.03), Phase::Aqueous);
    solver.equilibrate(&mut vessel).unwrap();
    assert!((s_iv(&vessel) - 0.01).abs() < 1e-9, "{vessel:?}");
    assert_eq!(amount(&vessel, "SO2", Phase::Gas), 0.0);
    // Without a headspace sink the neutral share is larger than sealed.
    assert!(amount(&vessel, "SO2", Phase::Aqueous) > 0.0075);
}
