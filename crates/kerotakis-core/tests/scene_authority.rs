use kerotakis_core::authority::{
    MotionPolicy, ReconcileError, TransferDestination, TransferProposal, TransferReconciler,
};
use kerotakis_core::{Bench, Event, Moles, Operator, SpeciesId, VesselId};

fn run(checkpoints: &[f64], policy: MotionPolicy) -> (Bench, f64) {
    let mut bench = Bench::new();
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(10.0),
            at: None,
        })
        .unwrap();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    let mut reconciler = TransferReconciler::new(VesselId(0), VesselId(1), 0x070);
    for &target in checkpoints {
        let proposal = TransferProposal {
            from: VesselId(0),
            to: TransferDestination::Vessel {
                vessel: VesselId(1),
            },
            cumulative_fraction: target,
            replay_seed: 0x070,
        };
        if let Some(op) = reconciler.propose(&proposal).unwrap() {
            let events = bench.step(op).unwrap();
            reconciler.reconcile(&events).unwrap();
        }
    }
    // Policy controls frames only, never operator compilation or reconciliation.
    let _paint = policy.paints_intermediate_frames();
    (bench, reconciler.committed_fraction())
}

fn water(bench: &Bench, vessel: VesselId) -> f64 {
    bench
        .vessel(vessel)
        .unwrap()
        .contents
        .iter()
        .filter(|p| p.species.0 == "water")
        .map(|p| p.moles.0)
        .sum()
}

fn proposal(target: f64) -> TransferProposal {
    TransferProposal {
        from: VesselId(0),
        to: TransferDestination::Vessel {
            vessel: VesselId(1),
        },
        cumulative_fraction: target,
        replay_seed: 0x070,
    }
}

#[test]
fn refusal_bad_targets_and_wrong_receipts_never_advance_authority() {
    let mut reconciler = TransferReconciler::new(VesselId(0), VesselId(1), 0x070);
    assert_eq!(
        reconciler.reconcile(&[]),
        Err(ReconcileError::UnexpectedReceipt)
    );
    assert_eq!(reconciler.committed_fraction(), 0.0);
    assert_eq!(
        reconciler.propose(&proposal(f64::NAN)),
        Err(ReconcileError::BadFraction)
    );
    assert_eq!(
        reconciler.propose(&proposal(1.01)),
        Err(ReconcileError::BadFraction)
    );

    reconciler.propose(&proposal(0.4)).unwrap();
    assert_eq!(
        reconciler.propose(&proposal(0.5)),
        Err(ReconcileError::AwaitingReceipt)
    );
    assert_eq!(
        reconciler.reconcile(&[Event::Transferred {
            from: VesselId(0),
            to: VesselId(1),
            fraction: 0.3,
        }]),
        Err(ReconcileError::UnexpectedReceipt)
    );
    assert_eq!(reconciler.committed_fraction(), 0.0);
    reconciler
        .reconcile(&[Event::Transferred {
            from: VesselId(0),
            to: VesselId(1),
            fraction: 0.4,
        }])
        .unwrap();
    assert_eq!(
        reconciler.propose(&proposal(0.39)),
        Err(ReconcileError::BadFraction)
    );
    assert_eq!(reconciler.committed_fraction(), 0.4);
}

#[test]
fn replay_seed_and_proposal_shape_survive_host_serialization() {
    let original = proposal(0.625);
    let json = serde_json::to_string(&original).unwrap();
    let replayed: TransferProposal = serde_json::from_str(&json).unwrap();
    assert_eq!(replayed, original);
    let mut a = TransferReconciler::new(VesselId(0), VesselId(1), 0x070);
    let mut b = TransferReconciler::new(VesselId(0), VesselId(1), 0x070);
    let a = a.propose(&original).unwrap();
    let b = b.propose(&replayed).unwrap();
    assert_eq!(a, b);
}

#[test]
fn failed_steps_can_be_cancelled_and_serialized_state_is_validated() {
    let mut reconciler = TransferReconciler::new(VesselId(0), VesselId(1), 0x070);
    reconciler.propose(&proposal(0.4)).unwrap();
    assert_eq!(
        reconciler.reconcile(&[]),
        Err(ReconcileError::MissingReceipt)
    );
    assert!(reconciler.cancel_pending());
    assert_eq!(reconciler.committed_fraction(), 0.0);
    assert!(reconciler.propose(&proposal(0.5)).unwrap().is_some());

    let json = serde_json::to_string(&reconciler).unwrap();
    assert_eq!(
        json,
        r#"{"from":0,"to":1,"replay_seed":112,"committed_fraction":0.0,"pending_fraction":0.5}"#
    );
    assert!(serde_json::from_str::<TransferReconciler>(
        r#"{"from":0,"to":1,"replay_seed":112,"committed_fraction":2.0,"pending_fraction":null}"#,
    )
    .is_err());
}

#[test]
fn duplicate_matching_receipts_are_rejected() {
    let mut reconciler = TransferReconciler::new(VesselId(0), VesselId(1), 0x070);
    reconciler.propose(&proposal(0.4)).unwrap();
    let receipt = Event::Transferred {
        from: VesselId(0),
        to: VesselId(1),
        fraction: 0.4,
    };
    assert_eq!(
        reconciler.reconcile(&[receipt.clone(), receipt]),
        Err(ReconcileError::UnexpectedReceipt)
    );
    assert_eq!(reconciler.committed_fraction(), 0.0);
}

#[test]
fn frame_rate_reduced_motion_and_headless_do_not_change_transferred_moles() {
    let (sixty_fps, _) = run(&[0.1, 0.2, 0.3, 0.4], MotionPolicy::Animated);
    let (twelve_fps, _) = run(&[0.2, 0.4], MotionPolicy::Animated);
    let (reduced, _) = run(&[0.4], MotionPolicy::ReducedMotion);
    let (headless, _) = run(&[0.4], MotionPolicy::Headless);
    for bench in [&sixty_fps, &twelve_fps, &reduced, &headless] {
        assert!((water(bench, VesselId(1)) - 4.0).abs() < 1e-12);
        assert!((water(bench, VesselId(0)) - 6.0).abs() < 1e-12);
    }
}

#[test]
fn interrupted_pour_reconciles_exactly_and_resumes_without_double_transfer() {
    let (interrupted, committed) = run(&[0.07, 0.23, 0.37], MotionPolicy::Background);
    assert!((committed - 0.37).abs() < 1e-15);
    assert!((water(&interrupted, VesselId(1)) - 3.7).abs() < 1e-12);
    let (single_endpoint, _) = run(&[0.37], MotionPolicy::Headless);
    assert!(
        (water(&single_endpoint, VesselId(1)) - water(&interrupted, VesselId(1))).abs() < 1e-12
    );
}
