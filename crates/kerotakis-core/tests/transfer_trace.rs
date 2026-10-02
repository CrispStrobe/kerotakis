//! Physical transfer inventory must not use a display-size cutoff.
use kerotakis_core::{authority::SpillDestination, vessel::UnresolvedMaterialPortion, *};

const TRACE: f64 = 1e-18;

fn fixture() -> Bench {
    let mut bench = Bench::new();
    let oil = material::lookup("olive_oil", None).expect("reviewed liquid recipe");
    for id in 0..3 {
        if id > 0 {
            bench.vessels.push(Vessel::new(VesselId(id), "beaker"));
        }
        if id == 2 {
            continue;
        }
        let v = &mut bench.vessels[id];
        let scale = (id + 1) as f64;
        v.deposit(SpeciesId::new("water"), Moles(scale), Phase::Liquid);
        v.deposit(SpeciesId::new("NaCl"), Moles(scale * TRACE), Phase::Aqueous);
        v.deposit(
            SpeciesId::new("Fe"),
            Moles(2.0 * scale * TRACE),
            Phase::Solid,
        );
        v.unresolved_materials.push(UnresolvedMaterialPortion {
            material: oil.canonical_key.clone(),
            recipe_id: oil.id.clone(),
            recipe_version: oil.version,
            basis: oil.basis,
            amount: 3.0 * scale * TRACE,
            enzyme_hydrolysis: None,
            protein_denatured_fraction: 0.0,
        });
    }
    bench
}

fn step(bench: &mut Bench, op: Operator) -> Vec<Event> {
    // Isolate the real operator path from unrelated chemistry. Numerical
    // proposal validation, safety and event publication still run normally.
    bench
        .step_with(op, &mut SolverStack::new(vec![]), &PermissiveScreen)
        .unwrap()
}

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= expected.abs() * 1e-14,
        "inventory {actual:e} != {expected:e}"
    );
}

fn check_vessel(bench: &Bench, id: usize, liquid: f64, solid: f64) {
    let v = &bench.vessels[id];
    close(v.moles_of(&SpeciesId::new("NaCl")).0, liquid * TRACE);
    close(v.moles_of(&SpeciesId::new("Fe")).0, solid * 2.0 * TRACE);
    close(
        v.unresolved_materials.iter().map(|p| p.amount).sum(),
        liquid * 3.0 * TRACE,
    );
}

#[test]
fn zero_and_fractional_decants_keep_moved_residual_and_unrelated_traces() {
    for fraction in [0.0, 0.25, 1.0] {
        let mut bench = fixture();
        step(
            &mut bench,
            Operator::Decant {
                from: VesselId(0),
                to: VesselId(2),
                fraction,
            },
        );
        check_vessel(&bench, 0, 1.0 - fraction, 1.0);
        check_vessel(&bench, 1, 2.0, 2.0);
        check_vessel(&bench, 2, fraction, 0.0);
    }
}

#[test]
fn mixing_two_streams_keeps_every_positive_trace_in_each_reservoir() {
    for (fraction_a, fraction_b) in [(0.0, 0.0), (0.25, 0.75), (1.0, 1.0)] {
        let mut bench = fixture();
        step(
            &mut bench,
            Operator::Mix {
                a: VesselId(0),
                b: VesselId(1),
                into: VesselId(2),
                fraction_a,
                fraction_b,
            },
        );
        check_vessel(&bench, 0, 1.0 - fraction_a, 1.0);
        check_vessel(&bench, 1, 2.0 * (1.0 - fraction_b), 2.0);
        check_vessel(&bench, 2, fraction_a + 2.0 * fraction_b, 0.0);
    }
}

#[test]
fn spill_then_partial_recovery_keeps_trace_stock_and_can_recover_the_remainder() {
    let mut bench = fixture();
    let destination = SpillDestination::Bench {
        zone: "trace".into(),
    };
    step(
        &mut bench,
        Operator::Spill {
            from: VesselId(0),
            destination: destination.clone(),
            fraction: 0.5,
            replay_seed: 1,
        },
    );
    check_vessel(&bench, 0, 0.5, 1.0);
    step(
        &mut bench,
        Operator::RecoverSpill {
            destination: destination.clone(),
            to: VesselId(2),
            fraction: 0.5,
        },
    );
    check_vessel(&bench, 2, 0.25, 0.0);
    let spill = bench
        .spill(&destination)
        .expect("positive remainder remains recoverable");
    close(
        spill
            .contents
            .iter()
            .filter(|p| p.species.0 == "NaCl")
            .map(|p| p.moles.0)
            .sum(),
        0.25 * TRACE,
    );
    close(
        spill.unresolved_materials.iter().map(|p| p.amount).sum(),
        0.75 * TRACE,
    );
    step(
        &mut bench,
        Operator::RecoverSpill {
            destination: destination.clone(),
            to: VesselId(2),
            fraction: 1.0,
        },
    );
    check_vessel(&bench, 2, 0.5, 0.0);
    assert!(bench.spill(&destination).is_none());
}

#[test]
fn discard_accounts_for_positive_trace_in_waste_and_numeric_event() {
    let mut bench = fixture();
    let events = step(
        &mut bench,
        Operator::Discard {
            vessel: VesselId(0),
        },
    );
    check_vessel(&bench, 0, 0.0, 0.0);
    let waste = bench.spill(&SpillDestination::Waste).unwrap();
    for (species, expected) in [("NaCl", TRACE), ("Fe", 2.0 * TRACE)] {
        close(
            waste
                .contents
                .iter()
                .filter(|p| p.species.0 == species)
                .map(|p| p.moles.0)
                .sum(),
            expected,
        );
    }
    close(
        waste.unresolved_materials.iter().map(|p| p.amount).sum(),
        3.0 * TRACE,
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::Discarded { species: portions, .. }
        if portions.iter().any(|p| p.species.0 == "Fe" && p.moles.0 == 2.0 * TRACE))));
}

#[test]
fn broken_vessel_transfers_solid_and_liquid_traces_to_its_spill() {
    let mut bench = fixture();
    let destination = SpillDestination::Floor {
        zone: "trace".into(),
    };
    step(
        &mut bench,
        Operator::Impact {
            vessel: VesselId(0),
            impulse_ns: 2.0,
            destination_if_broken: destination.clone(),
            replay_seed: 1,
        },
    );
    assert!(bench.is_broken(VesselId(0)));
    let spill = bench
        .spill(&destination)
        .expect("broken inventory is held outside the vessel");
    for (species, expected) in [("NaCl", TRACE), ("Fe", 2.0 * TRACE)] {
        close(
            spill
                .contents
                .iter()
                .filter(|p| p.species.0 == species)
                .map(|p| p.moles.0)
                .sum(),
            expected,
        );
    }
    close(
        spill.unresolved_materials.iter().map(|p| p.amount).sum(),
        3.0 * TRACE,
    );
}

#[test]
fn phase_changes_remove_a_total_once_and_preserve_unrelated_traces() {
    for (phase, temperature, product) in [
        (Phase::Liquid, 260.0, Phase::Solid),
        (Phase::Solid, 280.0, Phase::Liquid),
        (Phase::Liquid, 380.0, Phase::Gas),
    ] {
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.temperature = Kelvin(temperature);
        v.headspace = Headspace::Sealed {
            volume: Liters(1.0),
        };
        // Persisted inventories may contain multiple portions of a species.
        // Conservation must not subtract the total from each portion.
        v.deposit(SpeciesId::new("water"), Moles(0.5), phase);
        let second = v.contents[0].clone();
        v.contents.push(second);
        v.deposit(SpeciesId::new("Fe"), Moles(2.0 * TRACE), Phase::Solid);
        let events = SolverStack::new(vec![Box::new(StateEquilibrator)])
            .equilibrate(&mut v)
            .unwrap();
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::SolverFailed { .. })),
            "{events:?}"
        );
        assert!(
            v.contents
                .iter()
                .any(|p| p.species.0 == "water" && p.phase == product && p.moles.0 > 0.0),
            "transition at {temperature}K must actually happen"
        );
        close(v.moles_of(&SpeciesId::new("water")).0, 1.0);
        close(v.moles_of(&SpeciesId::new("Fe")).0, 2.0 * TRACE);
    }
}

#[test]
fn filtration_preserves_source_gas_and_transfers_only_liquid_and_aqueous_stock() {
    for gas_amount in [TRACE, 0.01] {
        let mut bench = fixture();
        bench.vessels[0].deposit(SpeciesId::new("N2"), Moles(gas_amount), Phase::Gas);
        let events = step(
            &mut bench,
            Operator::Filter {
                from: VesselId(0),
                to: VesselId(2),
            },
        );
        assert!(events.iter().any(|e| matches!(e, Event::Filtered { .. })));
        close(
            bench.vessels[0].moles_of(&SpeciesId::new("N2")).0,
            gas_amount,
        );
        close(bench.vessels[2].moles_of(&SpeciesId::new("N2")).0, 0.0);
        close(
            bench.vessels[0].moles_of(&SpeciesId::new("Fe")).0,
            2.0 * TRACE,
        );
        close(bench.vessels[0].moles_of(&SpeciesId::new("water")).0, 0.0);
        close(bench.vessels[2].moles_of(&SpeciesId::new("water")).0, 1.0);
        close(bench.vessels[2].moles_of(&SpeciesId::new("NaCl")).0, TRACE);
    }
}
