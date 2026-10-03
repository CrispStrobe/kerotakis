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
    // Hold concentration fixed while crossing the former phase threshold.
    // A positive solid, however small, excludes a homogeneous-solution
    // projection back to the complete input aqueous totals.
    for dose in [1e-12, 1e-14] {
        let mut vessel = Vessel::new(VesselId(0), "scaled silver chloride precipitation");
        vessel.deposit(SpeciesId::new("water"), Moles(55.51 * dose), Phase::Liquid);
        vessel.deposit(SpeciesId::new("Ag+"), Moles(dose), Phase::Aqueous);
        vessel.deposit(SpeciesId::new("Cl-"), Moles(dose), Phase::Aqueous);
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
            let ledger = kerotakis_core::ledger::ConservedLedger::from_vessel(&vessel);
            for element in ["Ag", "Cl"] {
                let actual = ledger.elements.get(element).copied().unwrap_or(0.0);
                assert!(
                    (actual / dose - 1.0).abs() < 1e-8,
                    "dose {dose:e}, pass {pass}: {element} budget={actual:e}"
                );
            }
        }
    }
}
