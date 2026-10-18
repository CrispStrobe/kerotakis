//! Chemical equilibrium must not replace a paid latent-heat phase fraction.
use kerotakis_cea::ThermalEquilibrator;
use kerotakis_core::{phase_route, *};

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(phase_route::PhaseRouteEquilibrator),
        Box::new(ThermalEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn step(bench: &mut Bench, stack: &mut SolverStack, op: Operator) -> Vec<Event> {
    let events = bench.step_with(op, stack, &PermissiveScreen).expect("step");
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::SolverFailed { .. })),
        "the phase control must actually solve: {events:?}"
    );
    events
}

fn amount(vessel: &Vessel, key: &str, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn phase_change(events: &[Event], key: &str) -> f64 {
    events
        .iter()
        .filter_map(|event| match event {
            Event::StateChanged {
                species,
                from,
                to,
                moles: Some(n),
                ..
            } if species.0 == key => match (from, to) {
                (Phase::Solid, Phase::Liquid) => Some(n.0),
                (Phase::Liquid, Phase::Solid) => Some(-n.0),
                _ => None,
            },
            _ => None,
        })
        .sum()
}

fn hot_sample(key: &str) -> (Bench, SolverStack, f64, f64) {
    let mut bench = Bench::new();
    let mut stack = stack();
    let data = species::lookup_key(key).expect("registry species");
    let melting = data
        .transitions
        .expect("phase transitions")
        .melting_k
        .expect("melting point");
    let latent = phase_route::fusion_enthalpy(key).expect("fusion enthalpy");
    step(
        &mut bench,
        &mut stack,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(key),
            moles: Moles(0.1),
            at: Some(Kelvin(melting - 20.0)),
        },
    );
    assert!((amount(&bench.vessels[0], key, Phase::Solid) - 0.1).abs() < 1e-12);
    (bench, stack, melting, latent)
}

#[test]
fn paid_partial_melt_survives_chemical_settlement_and_repeated_solve() {
    for key in ["Zn", "Pb", "naphthalene"] {
        let (mut bench, mut stack, melting, latent) = hot_sample(key);
        let vessel = &bench.vessels[0];
        let sensible = vessel.energy_between(vessel.temperature.0, melting);
        let energy = sensible + 0.05 * latent;
        let events = step(
            &mut bench,
            &mut stack,
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(energy),
                source: None,
            },
        );
        let vessel = &bench.vessels[0];
        let liquid = amount(vessel, key, Phase::Liquid);
        let solid = amount(vessel, key, Phase::Solid);
        assert!((liquid - 0.05).abs() < 1e-9, "{key}: {vessel:?}");
        assert!((solid - 0.05).abs() < 1e-9, "{key}: {vessel:?}");
        assert!((vessel.temperature.0 - melting).abs() < 1e-7);
        assert!((phase_change(&events, key) - liquid).abs() < 1e-10);
        assert!((sensible + liquid * latent - energy).abs() < 1e-5);

        // Re-solving is unchanged; advancing the clock is not, because the
        // clock includes an explicit ambient heat-exchange boundary.
        for _ in 0..3 {
            let events = stack.equilibrate(&mut bench.vessels[0]).expect("settle");
            assert!(
                phase_change(&events, key).abs() < 1e-12,
                "{key}: {events:?}"
            );
            assert!((amount(&bench.vessels[0], key, Phase::Liquid) - liquid).abs() < 1e-10);
            assert!((amount(&bench.vessels[0], key, Phase::Solid) - solid).abs() < 1e-10);
        }
    }
}

#[test]
fn below_and_above_the_latent_budget_remain_distinct() {
    for key in ["Zn", "Pb"] {
        for fully_molten in [false, true] {
            let (mut bench, mut stack, melting, latent) = hot_sample(key);
            let vessel = &bench.vessels[0];
            let sensible = vessel.energy_between(vessel.temperature.0, melting);
            let energy = if fully_molten {
                sensible + 0.11 * latent
            } else {
                sensible / 2.0
            };
            step(
                &mut bench,
                &mut stack,
                Operator::Heat {
                    vessel: VesselId(0),
                    energy: Joules(energy),
                    source: None,
                },
            );
            let vessel = &bench.vessels[0];
            let expected = if fully_molten {
                Phase::Liquid
            } else {
                Phase::Solid
            };
            assert!(
                (amount(vessel, key, expected) - 0.1).abs() < 1e-9,
                "{key}: {vessel:?}"
            );
        }
    }
}

#[test]
fn reverse_cooling_and_pouring_retain_paid_phase_ownership() {
    let key = "Zn";
    let (mut bench, mut stack, melting, latent) = hot_sample(key);
    let sensible = bench.vessels[0].energy_between(bench.vessels[0].temperature.0, melting);
    step(
        &mut bench,
        &mut stack,
        Operator::Heat {
            vessel: VesselId(0),
            energy: Joules(sensible + 0.05 * latent),
            source: None,
        },
    );
    let before = amount(&bench.vessels[0], key, Phase::Liquid);
    let events = step(
        &mut bench,
        &mut stack,
        Operator::Cool {
            vessel: VesselId(0),
            energy: Joules(0.025 * latent),
        },
    );
    let liquid = amount(&bench.vessels[0], key, Phase::Liquid);
    assert!(
        (liquid - 0.025).abs() < 1e-9,
        "cooling must retain its paid frozen fraction: liquid={liquid}, events={events:?}, vessel={:?}",
        bench.vessels[0]
    );
    assert!((phase_change(&events, key) - (liquid - before)).abs() < 1e-10);
    step(&mut bench, &mut stack, Operator::NewVessel { kind: None });
    step(
        &mut bench,
        &mut stack,
        Operator::Decant {
            from: VesselId(0),
            to: VesselId(1),
            fraction: 0.5,
        },
    );
    assert!((amount(&bench.vessels[0], key, Phase::Solid) - 0.075).abs() < 1e-9);
    assert!((amount(&bench.vessels[0], key, Phase::Liquid) - 0.0125).abs() < 1e-9);
    assert!((amount(&bench.vessels[1], key, Phase::Liquid) - 0.0125).abs() < 1e-9);
    assert!(bench
        .vessels
        .iter()
        .flat_map(|v| &v.contents)
        .all(|p| p.species.0 == key));
}

#[test]
fn a_real_oxidation_is_not_a_phase_only_handoff() {
    let mut bench = Bench::new();
    let mut stack = stack();
    step(
        &mut bench,
        &mut stack,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("Mg"),
            moles: Moles(0.01),
            at: None,
        },
    );
    let events = step(
        &mut bench,
        &mut stack,
        Operator::Ignite {
            vessel: VesselId(0),
        },
    );
    assert!(bench.vessels[0].moles_of(&SpeciesId::new("MgO")).0 > 0.009);
    assert!(bench.vessels[0].moles_of(&SpeciesId::new("Mg")).0 < 1e-6);
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::Consumed { species, .. } if species.0 == "Mg")));
}
