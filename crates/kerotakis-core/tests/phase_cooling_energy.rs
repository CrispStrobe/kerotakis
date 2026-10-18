//! A latent transition must not create sensible energy when phase Cp differs.
use kerotakis_core::{phase_route, *};

fn amount(v: &Vessel, key: &str, phase: Phase) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

#[test]
fn partial_freezing_closes_the_transition_referenced_energy_for_different_phases() {
    for key in ["Zn", "Pb", "naphthalene", "water"] {
        let data = species::lookup_key(key).expect("registry");
        let melting = data
            .transitions
            .expect("transitions")
            .melting_k
            .expect("melting");
        // Water belongs to the solvent state model, not the separate
        // molecular/metal phase-route table. Exercise its actual owner too.
        let latent = if key == "water" {
            states::WATER_H_FUS
        } else {
            phase_route::fusion_enthalpy(key).expect("fusion")
        };
        let mut bench = Bench::new();
        let mut stack = SolverStack::new(vec![
            Box::new(phase_route::PhaseRouteEquilibrator),
            Box::new(StateEquilibrator),
        ]);
        // Prepared coexistence is the input, not the result. The actual
        // cool operator and phase route must perform and retain the debit.
        let v = &mut bench.vessels[0];
        v.temperature = Kelvin(melting);
        v.deposit(SpeciesId::new(key), Moles(0.05), Phase::Solid);
        v.deposit(SpeciesId::new(key), Moles(0.05), Phase::Liquid);
        let before = amount(v, key, Phase::Liquid) * latent;
        let removed = 0.025 * latent;
        let events = bench
            .step_with(
                Operator::Cool {
                    vessel: VesselId(0),
                    energy: Joules(removed),
                },
                &mut stack,
                &PermissiveScreen,
            )
            .expect("cool");
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })));
        let v = &bench.vessels[0];
        let after =
            v.energy_between(melting, v.temperature.0) + amount(v, key, Phase::Liquid) * latent;
        assert!(
            (before - after - removed).abs() < 1e-7,
            "{key}: removed={removed}, before={before}, after={after}, vessel={v:?}"
        );
        assert!(
            (amount(v, key, Phase::Liquid) - 0.025).abs() < 1e-10,
            "{key}: {v:?}"
        );
        assert!(
            (amount(v, key, Phase::Solid) - 0.075).abs() < 1e-10,
            "{key}: {v:?}"
        );
        assert!((v.temperature.0 - melting).abs() < 1e-8, "{key}: {v:?}");
        for _ in 0..3 {
            stack
                .equilibrate(&mut bench.vessels[0])
                .expect("unchanged settlement");
            assert!((amount(&bench.vessels[0], key, Phase::Liquid) - 0.025).abs() < 1e-10);
        }
    }
}
