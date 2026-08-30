use kerotakis_core::authority::SpillDestination;
use kerotakis_core::{Bench, Moles, Operator, SpeciesId, VesselId};

fn bench_spill() -> SpillDestination {
    SpillDestination::Bench {
        zone: "west".to_string(),
    }
}

fn add_water(bench: &mut Bench, vessel: VesselId, moles: f64) {
    bench
        .step(Operator::Add {
            vessel,
            species: SpeciesId::new("water"),
            moles: Moles(moles),
            at: None,
        })
        .expect("add water");
}

fn water_moles(bench: &Bench, vessel: VesselId) -> f64 {
    bench
        .vessel(vessel)
        .expect("vessel")
        .contents
        .iter()
        .filter(|portion| portion.species.0 == "water")
        .map(|portion| portion.moles.0)
        .sum()
}

#[test]
fn old_saves_migrate_with_empty_spill_and_breakage_state() {
    let mut value = serde_json::to_value(Bench::new()).expect("serialize bench");
    let object = value.as_object_mut().expect("bench object");
    object.remove("spills");
    object.remove("broken_vessels");

    let restored: Bench = serde_json::from_value(value).expect("load pre-BRD-073 save");
    assert!(restored.spills.is_empty());
    assert!(restored.broken_vessels.is_empty());
}

#[test]
fn undo_restore_then_recover_cannot_duplicate_spilled_stock() {
    let destination = bench_spill();
    let mut bench = Bench::new();
    add_water(&mut bench, VesselId(0), 1.0);
    bench
        .step(Operator::Spill {
            from: VesselId(0),
            destination: destination.clone(),
            fraction: 0.6,
            replay_seed: 73,
        })
        .expect("spill");
    bench
        .step(Operator::NewVessel {
            kind: Some("beaker".to_string()),
        })
        .expect("replacement receiver");

    let undo_point = serde_json::to_string(&bench).expect("snapshot before recovery");
    bench
        .step(Operator::RecoverSpill {
            destination: destination.clone(),
            to: VesselId(1),
            fraction: 1.0,
        })
        .expect("first recovery");
    let first_result = serde_json::to_value(&bench).expect("first result");

    let mut restored: Bench = serde_json::from_str(&undo_point).expect("undo restore");
    restored
        .step(Operator::RecoverSpill {
            destination,
            to: VesselId(1),
            fraction: 1.0,
        })
        .expect("replayed recovery");

    assert_eq!(serde_json::to_value(&restored).unwrap(), first_result);
    assert!((water_moles(&restored, VesselId(0)) - 0.4).abs() < 1e-12);
    assert!((water_moles(&restored, VesselId(1)) - 0.6).abs() < 1e-12);
    assert!(
        restored.spills.is_empty(),
        "stock left only in the receiver"
    );
}

#[test]
fn saved_operator_log_replays_breakage_spill_and_replacement_exactly() {
    let destination = bench_spill();
    let mut original = Bench::new();
    add_water(&mut original, VesselId(0), 0.75);
    original
        .step(Operator::Impact {
            vessel: VesselId(0),
            impulse_ns: 2.0,
            destination_if_broken: destination.clone(),
            replay_seed: 731,
        })
        .expect("breaking impact");
    original
        .step(Operator::NewVessel {
            kind: Some("beaker".to_string()),
        })
        .expect("replacement glassware");

    let saved = serde_json::to_string(&original).expect("save bench");
    let loaded: Bench = serde_json::from_str(&saved).expect("load bench");
    assert!(loaded.is_broken(VesselId(0)));
    assert_eq!(loaded.vessels.last().unwrap().id, VesselId(1));
    assert!((loaded.spill(&destination).unwrap().contents[0].moles.0 - 0.75).abs() < 1e-12);

    let operators = original
        .log
        .iter()
        .map(|entry| entry.operator.clone())
        .collect::<Vec<_>>();
    let mut replay = Bench::new();
    for operator in operators {
        replay.step(operator).expect("operator replay");
    }
    assert_eq!(
        serde_json::to_value(replay).unwrap(),
        serde_json::to_value(original).unwrap()
    );
}
