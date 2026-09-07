//! Native MIX must decline coupled electron ledgers before mutating the target.
//! The ordinary equilibrium owner then computes the merged inventory.
#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn solution(id: usize, solutes: &[(&str, f64)]) -> Vessel {
    let mut vessel = Vessel::new(VesselId(id), "mix boundary");
    vessel.thermal_mode = ThermalMode::Thermostatted(Kelvin::STANDARD);
    vessel.contents.push(Portion {
        species: SpeciesId::new("water"),
        moles: Moles(2.775),
        phase: Phase::Liquid,
    });
    vessel
        .contents
        .extend(solutes.iter().map(|(species, moles)| Portion {
            species: SpeciesId::new(species),
            moles: Moles(*moles),
            phase: Phase::Aqueous,
        }));
    vessel
}

fn merge(a: &Vessel, fa: f64, b: &Vessel, fb: f64) -> Vessel {
    let mut target = Vessel::new(VesselId(2), "merged inventory");
    target.thermal_mode = ThermalMode::Thermostatted(Kelvin::STANDARD);
    for (source, fraction) in [(a, fa), (b, fb)] {
        for portion in &source.contents {
            if let Some(existing) = target
                .contents
                .iter_mut()
                .find(|p| p.species == portion.species && p.phase == portion.phase)
            {
                existing.moles.0 += portion.moles.0 * fraction;
            } else {
                let mut added = portion.clone();
                added.moles.0 *= fraction;
                target.contents.push(added);
            }
        }
    }
    target
}

#[test]
fn coupled_mix_declines_without_mutation_and_normal_solve_balances_electrons() {
    let mut eq = PhreeqcEquilibrator::new().expect("native engine");
    let a = solution(
        0,
        &[
            ("H+", 0.005),
            ("Cl-", 0.005),
            ("Fe+2", 0.005),
            ("SO4-2", 0.005),
        ],
    );
    let b = solution(
        1,
        &[
            ("H+", 0.005),
            ("Cl-", 0.005),
            ("K+", 0.0002),
            ("MnO4-", 0.0002),
        ],
    );
    for (fa, fb) in [(1.0, 1.0), (0.5, 1.0), (1.0, 0.5)] {
        let mut target = merge(&a, fa, &b, fb);
        let before = serde_json::to_value(&target).unwrap();
        assert!(eq.mix(&mut target, &a, fa, &b, fb).is_none());
        assert_eq!(serde_json::to_value(&target).unwrap(), before);

        let events = eq
            .equilibrate(&mut target)
            .expect("coupled direct fallback");
        let result = target.solution.as_ref().expect("computed solution");
        assert!(result.pe.is_some(),
            "fractions {fa}/{fb}: require a computed coupled root, not as-added fallback: {events:?}");
        let kg = result.solvent_kg.expect("native solvent mass");
        let ledger: f64 = result
            .redox
            .iter()
            .filter(|state| matches!(state.element.as_str(), "Fe" | "Mn"))
            .map(|state| state.oxidation as f64 * state.molality * kg)
            .sum();
        let expected = 2.0 * 0.005 * fa + 7.0 * 0.0002 * fb;
        // Match the bounded root's existing electron residual, not exact
        // floating-point equality or a hard-coded reaction product table.
        assert!(
            (ledger - expected).abs() < expected * 0.002,
            "fractions {fa}/{fb}: electron ledger {ledger}, expected {expected}"
        );
        let ferric: f64 = result
            .redox
            .iter()
            .filter(|state| state.element == "Fe" && state.oxidation == 3)
            .map(|state| state.molality * kg)
            .sum();
        let manganese_ii: f64 = result
            .redox
            .iter()
            .filter(|state| state.element == "Mn" && state.oxidation == 2)
            .map(|state| state.molality * kg)
            .sum();
        assert!(
            manganese_ii > 0.98 * 0.0002 * fb,
            "fractions {fa}/{fb}: MnII={manganese_ii}, solution={result:?}, events={events:?}"
        );
        assert!((ferric - 5.0 * 0.0002 * fb).abs() < 0.02 * 0.005 * fa);
    }
}

#[test]
fn uncoupled_native_mix_remains_available() {
    let mut eq = PhreeqcEquilibrator::new().expect("native engine");
    let a = solution(0, &[("Na+", 0.001), ("Cl-", 0.001)]);
    let b = solution(1, &[("K+", 0.002), ("Cl-", 0.002)]);
    let mut target = merge(&a, 1.0, &b, 0.5);
    eq.mix(&mut target, &a, 1.0, &b, 0.5)
        .expect("uncoupled MIX supported")
        .expect("native MIX succeeds");
    assert!(target.solution.as_ref().is_some_and(|s| s.ph.is_finite()));
}
