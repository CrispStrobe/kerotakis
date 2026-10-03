//! Accepted trace doses remain material across repeated aqueous solves.
#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

#[test]
fn trace_acid_and_salt_spectators_survive_repeated_equilibration() {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    // Exercise both sides and the exact old 1e-12 mol cutoff, with acid
    // and neutral salt inputs. A dose below display resolution is still
    // the authoritative element budget for the next solve.
    for ingredient in ["HCl", "NaCl"] {
        for dose in [1e-9, 2e-12, 1e-12, 1e-14] {
            let mut vessel = Vessel::new(VesselId(0), "trace spectator cell");
            vessel.deposit(SpeciesId::new("water"), Moles(5.53427699), Phase::Liquid);
            vessel.deposit(SpeciesId::new(ingredient), Moles(dose), Phase::Aqueous);
            for pass in 0..3 {
                solver.equilibrate(&mut vessel).unwrap_or_else(|error| {
                    panic!("{ingredient}, dose {dose:e}, pass {pass}: {error:?}")
                });
                let ledger = kerotakis_core::ledger::ConservedLedger::from_vessel(&vessel);
                for element in if ingredient == "NaCl" {
                    &["Na", "Cl"][..]
                } else {
                    &["Cl"][..]
                } {
                    let actual = ledger.elements.get(*element).copied().unwrap_or(0.0);
                    assert!(
                        (actual / dose - 1.0).abs() < 1e-8,
                        "{ingredient}, dose {dose:e}, pass {pass}: {element}={actual:e}"
                    );
                }
                let ph = vessel.solution.as_ref().expect("characterized solution").ph;
                assert!(
                    ph.is_finite() && (6.8..7.05).contains(&ph),
                    "trace-dose pH: {ph}"
                );
                // Change solvent scale before the next solve. Conserved
                // spectators must not vanish when their concentration falls.
                vessel.deposit(SpeciesId::new("water"), Moles(0.5), Phase::Liquid);
            }
        }
    }
}

#[test]
fn subpicomole_precipitation_never_duplicates_the_dissolved_element_budget() {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    // Neutral silver nitrate and sodium chloride at about 0.01 mol/kgw,
    // the established dilute precipitation preparation, scaled across the
    // former phase threshold. The primary positive control must complete:
    // a refusal cannot stand in for conservation of a real trace solid.
    let mut reference_fraction = None;
    for dose in [1e-3, 1e-12, 1e-14] {
        let mut vessel = Vessel::new(VesselId(0), "scaled silver chloride precipitation");
        vessel.thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
        vessel.deposit(SpeciesId::new("water"), Moles(5551.0 * dose), Phase::Liquid);
        vessel.deposit(SpeciesId::new("AgNO3"), Moles(dose), Phase::Aqueous);
        vessel.deposit(SpeciesId::new("NaCl"), Moles(dose), Phase::Aqueous);
        for pass in 0..3 {
            solver.equilibrate(&mut vessel).unwrap_or_else(|error| {
                panic!("precipitation dose {dose:e}, pass {pass}: {error:?}")
            });
            let solid: f64 = vessel
                .contents
                .iter()
                .filter(|p| p.species.0 == "AgCl" && p.phase == Phase::Solid)
                .map(|p| p.moles.0)
                .sum();
            assert!(
                solid > 0.9 * dose && solid <= dose * (1.0 + 1e-8),
                "expected a majority positive AgCl phase: dose={dose:e}, solid={solid:e}"
            );
            let fraction = solid / dose;
            if dose == 1e-3 && pass == 0 {
                reference_fraction = Some(fraction);
            }
            assert!((fraction - reference_fraction.unwrap()).abs() < 1e-6,
                "same-concentration normal/microscopic phase fraction: dose={dose:e}, pass={pass}, fraction={fraction}");
            let ledger = kerotakis_core::ledger::ConservedLedger::from_vessel(&vessel);
            for element in ["Ag", "Cl", "Na", "N"] {
                let actual = ledger.elements.get(element).copied().unwrap_or(0.0);
                assert!(
                    (actual / dose - 1.0).abs() < 1e-8,
                    "dose {dose:e}, pass {pass}: {element} budget={actual:e}"
                );
            }
        }
    }
}

#[test]
fn concentrated_bare_ion_trace_preparation_is_conservative_or_atomically_refused() {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    for dose in [1e-12, 1e-14] {
        let mut vessel = Vessel::new(VesselId(0), "concentrated bare-ion diagnostic");
        vessel.deposit(SpeciesId::new("water"), Moles(55.51 * dose), Phase::Liquid);
        vessel.deposit(SpeciesId::new("Ag+"), Moles(dose), Phase::Aqueous);
        vessel.deposit(SpeciesId::new("Cl-"), Moles(dose), Phase::Aqueous);
        let before = serde_json::to_value(&vessel).expect("before state");
        match solver.equilibrate(&mut vessel) {
            Ok(_) => {
                let ledger = kerotakis_core::ledger::ConservedLedger::from_vessel(&vessel);
                for element in ["Ag", "Cl"] {
                    let actual = ledger.elements.get(element).copied().unwrap_or(0.0);
                    assert!(
                        (actual / dose - 1.0).abs() < 1e-8,
                        "concentrated dose {dose:e}: {element} budget={actual:e}"
                    );
                }
                let solid: f64 = vessel
                    .contents
                    .iter()
                    .filter(|p| p.species.0 == "AgCl" && p.phase == Phase::Solid)
                    .map(|p| p.moles.0)
                    .sum();
                assert!(solid > 0.9 * dose && solid <= dose * (1.0 + 1e-8));
            }
            Err(SolveError::NotConverged { .. }) => {
                assert_eq!(
                    serde_json::to_value(&vessel).expect("after state"),
                    before,
                    "numerical refusal must preserve the complete persistent vessel state"
                );
                assert!(vessel.heat_input.is_none());
                assert!(!vessel.ignition_trial);
                assert!(vessel.ignition_feed_temperature.is_none());
            }
            Err(error) => panic!("unexpected refusal for concentrated trace sample: {error:?}"),
        }
    }
}

#[test]
fn native_mix_preserves_normal_and_microscopic_solution_budgets() {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    for kgw in [0.1, 1e-12, 1e-14] {
        let mut a = Vessel::new(VesselId(0), "first salt solution");
        let mut b = Vessel::new(VesselId(1), "second salt solution");
        for (vessel, water, salt) in [(&mut a, kgw, 0.01 * kgw), (&mut b, 2.0 * kgw, 0.08 * kgw)] {
            vessel.thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
            vessel.headspace = Headspace::Sealed {
                volume: Liters(water),
            };
            vessel.deposit(
                SpeciesId::new("water"),
                Moles(water * 1000.0 / kerotakis_core::constants::WATER_MOLAR_MASS_G_PER_MOL),
                Phase::Liquid,
            );
            vessel.deposit(SpeciesId::new("NaCl"), Moles(salt), Phase::Aqueous);
        }
        let mut mixed = Vessel::new(VesselId(2), "native mixed solution");
        mixed.thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
        mixed.headspace = Headspace::Sealed {
            volume: Liters(1.75 * kgw),
        };
        mixed.deposit(
            SpeciesId::new("water"),
            Moles(1.75 * kgw * 1000.0 / kerotakis_core::constants::WATER_MOLAR_MASS_G_PER_MOL),
            Phase::Liquid,
        );
        let expected = 0.0625 * kgw;
        mixed.deposit(SpeciesId::new("NaCl"), Moles(expected), Phase::Aqueous);
        let initial_ledger = kerotakis_core::ledger::ConservedLedger::from_vessel(&mixed);
        let mut direct = mixed.clone();
        solver
            .mix(&mut mixed, &a, 0.25, &b, 0.75)
            .expect("native MIX route")
            .unwrap_or_else(|error| panic!("MIX solvent {kgw:e}: {error:?}"));
        solver
            .equilibrate(&mut direct)
            .unwrap_or_else(|error| panic!("direct solvent {kgw:e}: {error:?}"));
        for vessel in [&mixed, &direct] {
            let ledger = kerotakis_core::ledger::ConservedLedger::from_vessel(vessel);
            for element in ["Na", "Cl"] {
                let actual = ledger.elements.get(element).copied().unwrap_or(0.0);
                assert!(
                    (actual / expected - 1.0).abs() < 1e-8,
                    "solvent {kgw:e}, {element}: {actual:e} versus {expected:e}"
                );
            }
            for element in ["H", "O"] {
                let actual = ledger.elements.get(element).copied().unwrap_or(0.0);
                let initial = initial_ledger.elements.get(element).copied().unwrap_or(0.0);
                assert!(
                    initial > 0.0 && (actual / initial - 1.0).abs() < 1e-8,
                    "closed solvent {kgw:e}, {element}: {actual:e} versus initial {initial:e}"
                );
            }
            assert!(
                (vessel.solution.as_ref().unwrap().solvent_kg.unwrap() / (1.75 * kgw) - 1.0).abs()
                    < 1e-6
            );
        }
        let native = mixed.solution.as_ref().unwrap();
        let ordinary = direct.solution.as_ref().unwrap();
        assert!((native.ph - ordinary.ph).abs() < 1e-5);
        assert!((native.ionic_strength / ordinary.ionic_strength - 1.0).abs() < 1e-6);
    }
}

#[test]
fn microscopic_native_cache_roundtrip_keeps_physical_amounts_without_double_scaling() {
    let mut builder = PhreeqcEquilibrator::new().expect("engine");
    let dose = 1e-14;
    let mut settled = Vessel::new(VesselId(0), "microscopic cached precipitation");
    settled.thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
    settled.deposit(SpeciesId::new("water"), Moles(5551.0 * dose), Phase::Liquid);
    settled.deposit(SpeciesId::new("AgNO3"), Moles(dose), Phase::Aqueous);
    settled.deposit(SpeciesId::new("NaCl"), Moles(dose), Phase::Aqueous);
    builder
        .equilibrate(&mut settled)
        .expect("required microscopic positive control");
    let mut expected = settled.clone();
    builder
        .equilibrate(&mut expected)
        .expect("settled native solve");
    let exported = builder.export_cache();
    let normalized: Vec<_> = exported
        .entries
        .iter()
        .filter(|entry| entry.key.contains("KERO_NATIVE_EXTENSIVE_SCALE"))
        .collect();
    assert!(
        !normalized.is_empty(),
        "microscopic solve must use native normalization"
    );
    for entry in normalized {
        let column = entry.rows[0]
            .iter()
            .position(|name| name == "mass_H2O")
            .expect("water header");
        let physical = entry.rows.last().unwrap()[column].parse::<f64>().unwrap();
        assert!(
            physical > 0.0 && physical < 1e-8,
            "cache contains physical water, not 1kg native water"
        );
    }
    let bytes = postcard::to_allocvec(&exported).expect("cache serialize");
    let loaded: kerotakis_phreeqc::CacheData =
        postcard::from_bytes(&bytes).expect("cache deserialize");
    let mut device = PhreeqcEquilibrator::new().expect("cold engine");
    assert_eq!(device.import_cache(loaded), exported.entries.len());
    device
        .equilibrate(&mut settled)
        .expect("physical-basis cache replay");
    assert_eq!(
        device.engine_calls(),
        0,
        "the imported solve must not fall back to native computation"
    );
    assert!(device.cache_hits() > 0);
    assert_eq!(
        serde_json::to_value(&settled.solution).unwrap(),
        serde_json::to_value(&expected.solution).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&settled.contents).unwrap(),
        serde_json::to_value(&expected.contents).unwrap()
    );
    let actual = kerotakis_core::ledger::ConservedLedger::from_vessel(&settled);
    let reference = kerotakis_core::ledger::ConservedLedger::from_vessel(&expected);
    for element in ["Ag", "Cl", "Na", "N", "H", "O"] {
        let got = actual.elements.get(element).copied().unwrap_or(0.0);
        let want = reference.elements.get(element).copied().unwrap_or(0.0);
        assert!(
            want > 0.0 && (got / want - 1.0).abs() < 1e-12,
            "cached {element}: {got:e} versus physical reference {want:e}"
        );
    }
}
