use kerotakis_core::*;

#[test]
fn an_empty_vessel_can_return_to_storage_without_renumbering_others() {
    let mut bench = Bench::new();
    bench
        .step(Operator::NewVessel {
            kind: Some("tube".into()),
        })
        .unwrap();
    bench.step(Operator::NewVessel { kind: None }).unwrap();

    let events = bench
        .step(Operator::RemoveVessel {
            vessel: VesselId(1),
        })
        .unwrap();
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::VesselRemoved { vessel } if *vessel == VesselId(1))));
    assert_eq!(
        bench.vessels.iter().map(|v| v.id.0).collect::<Vec<_>>(),
        vec![0, 2]
    );

    bench.step(Operator::NewVessel { kind: None }).unwrap();
    assert_eq!(
        bench.vessels.iter().map(|v| v.id.0).collect::<Vec<_>>(),
        vec![0, 2, 3]
    );
}

#[test]
fn removal_never_silently_discards_matter_or_the_last_receiver() {
    let mut bench = Bench::new();
    assert!(matches!(
        bench.step(Operator::RemoveVessel {
            vessel: VesselId(0)
        }),
        Err(BenchError::LastVessel)
    ));
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("water"),
            moles: Moles(1.0),
            at: None,
        })
        .unwrap();
    assert!(matches!(
        bench.step(Operator::RemoveVessel {
            vessel: VesselId(1)
        }),
        Err(BenchError::VesselNotEmpty(VesselId(1)))
    ));
    assert!(
        (bench
            .vessel(VesselId(1))
            .unwrap()
            .moles_of(&SpeciesId::new("water"))
            .0
            - 1.0)
            .abs()
            < 1e-12
    );
}
