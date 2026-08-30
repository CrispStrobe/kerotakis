use std::collections::BTreeMap;

use kerotakis_core::authority::{
    CollisionProposal, MotionPolicy, SpillDestination, SpillTransferReconciler,
    TransferDestination, TransferProposal,
};
use kerotakis_core::{Bench, ConservedLedger, Event, Kelvin, Moles, Operator, SpeciesId, VesselId};

fn totals(bench: &Bench) -> (BTreeMap<String, f64>, f64, f64) {
    let mut elements = BTreeMap::new();
    let mut mass = 0.0;
    let mut energy = 0.0;
    let ledgers = bench
        .vessels
        .iter()
        .map(ConservedLedger::from_vessel)
        .chain(
            bench
                .spills
                .iter()
                .map(|spill| ConservedLedger::from_vessel(&spill.as_vessel_probe())),
        );
    for ledger in ledgers {
        mass += ledger.mass;
        energy += ledger.energy;
        for (element, amount) in ledger.elements {
            *elements.entry(element).or_insert(0.0) += amount;
        }
    }
    (elements, mass, energy)
}

fn charged_bench() -> Bench {
    let mut bench = Bench::new();
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(5.0),
            at: Some(Kelvin::from_celsius(60.0)),
        })
        .unwrap();
    bench
}

#[test]
fn partial_spill_is_frame_policy_independent_and_closes_ledgers() {
    let destination = SpillDestination::Bench {
        zone: "left".into(),
    };
    let run = |checkpoints: &[f64], policy: MotionPolicy| {
        let mut bench = charged_bench();
        let before = totals(&bench);
        let mut reconciler = SpillTransferReconciler::new(VesselId(0), destination.clone(), 0x73);
        for target in checkpoints {
            let proposal = TransferProposal {
                from: VesselId(0),
                to: TransferDestination::Spill(destination.clone()),
                cumulative_fraction: *target,
                replay_seed: 0x73,
            };
            if let Some(operator) = reconciler.propose(&proposal).unwrap() {
                let events = bench.step(operator).unwrap();
                reconciler.reconcile(&events).unwrap();
            }
        }
        let _frames = policy.paints_intermediate_frames();
        assert_eq!(before.0, totals(&bench).0);
        assert!((before.1 - totals(&bench).1).abs() < 1e-10);
        assert!((before.2 - totals(&bench).2).abs() < 1e-8);
        bench
    };
    let animated = run(&[0.1, 0.2, 0.4], MotionPolicy::Animated);
    let headless = run(&[0.4], MotionPolicy::Headless);
    assert_eq!(
        serde_json::to_value(&animated.spills).unwrap(),
        serde_json::to_value(&headless.spills).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&animated.vessels).unwrap(),
        serde_json::to_value(&headless.vessels).unwrap()
    );
}

#[test]
fn breaking_moves_contents_once_and_recovery_cannot_duplicate_stock() {
    let mut bench = charged_bench();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    let destination = SpillDestination::Tray {
        tray: "primary".into(),
    };
    let before = totals(&bench);
    let proposal = CollisionProposal {
        vessel: VesselId(0),
        impulse_ns: 2.0,
        destination_if_broken: destination.clone(),
        replay_seed: 99,
    };
    let events = bench.step(proposal.to_operator().unwrap()).unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::ContainerBroken {
            vessel: VesselId(0),
            ..
        }
    )));
    assert!(bench.is_broken(VesselId(0)));
    assert!(bench.vessel(VesselId(0)).unwrap().contents.is_empty());
    assert_eq!(before.0, totals(&bench).0);
    assert!((before.1 - totals(&bench).1).abs() < 1e-10);
    assert!((before.2 - totals(&bench).2).abs() < 1e-8);
    assert!(bench.step(proposal.to_operator().unwrap()).is_err());

    bench
        .step(Operator::RecoverSpill {
            destination: destination.clone(),
            to: VesselId(1),
            fraction: 1.0,
        })
        .unwrap();
    assert!(bench.spill(&destination).is_none());
    assert!(bench
        .step(Operator::RecoverSpill {
            destination: destination.clone(),
            to: VesselId(1),
            fraction: 0.0,
        })
        .unwrap()
        .is_empty());
    assert_eq!(before.0, totals(&bench).0);
    assert!((before.1 - totals(&bench).1).abs() < 1e-10);
    assert!((before.2 - totals(&bench).2).abs() < 1e-8);
    assert!(bench
        .step(Operator::RecoverSpill {
            destination,
            to: VesselId(1),
            fraction: 1.0
        })
        .is_err());
}

#[test]
fn save_migration_replay_and_snapshot_undo_are_deterministic() {
    let original = charged_bench();
    let mut legacy = serde_json::to_value(&original).unwrap();
    legacy.as_object_mut().unwrap().remove("spills");
    legacy.as_object_mut().unwrap().remove("broken_vessels");
    let migrated: Bench = serde_json::from_value(legacy).unwrap();
    assert!(migrated.spills.is_empty());
    assert!(migrated.broken_vessels.is_empty());

    let destination = SpillDestination::Floor {
        zone: "north".into(),
    };
    let operator = Operator::Spill {
        from: VesselId(0),
        destination,
        fraction: 0.25,
        replay_seed: 7,
    };
    let snapshot = migrated.clone();
    let mut first = migrated;
    first.step(operator.clone()).unwrap();
    let encoded = serde_json::to_string(&first).unwrap();
    let restored: Bench = serde_json::from_str(&encoded).unwrap();
    let mut replayed = snapshot.clone();
    replayed.step(operator).unwrap();
    assert_eq!(
        serde_json::to_value(restored).unwrap(),
        serde_json::to_value(&replayed).unwrap()
    );
    assert_eq!(totals(&snapshot).0, totals(&replayed).0);
}

#[test]
fn zero_fraction_operations_and_non_breaking_impacts_are_state_noops() {
    let mut bench = charged_bench();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    let destination = SpillDestination::Bench {
        zone: "zero".into(),
    };
    let before = totals(&bench);
    assert!(bench
        .step(Operator::Spill {
            from: VesselId(0),
            destination: destination.clone(),
            fraction: 0.0,
            replay_seed: 1,
        })
        .unwrap()
        .is_empty());
    assert!(bench.spill(&destination).is_none());
    let events = bench
        .step(Operator::Impact {
            vessel: VesselId(0),
            impulse_ns: 0.25,
            destination_if_broken: destination,
            replay_seed: 2,
        })
        .unwrap();
    assert!(matches!(
        events.as_slice(),
        [Event::CollisionWithstood { .. }]
    ));
    assert!(!bench.is_broken(VesselId(0)));
    assert_eq!(before.0, totals(&bench).0);
    assert!((before.1 - totals(&bench).1).abs() < 1e-10);
    assert!((before.2 - totals(&bench).2).abs() < 1e-8);
}
