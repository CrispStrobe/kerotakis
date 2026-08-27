use kerotakis_core::script::parse_op;
use kerotakis_core::*;

#[test]
fn centrifuge_command_computes_rcf_and_separation_from_vessel_state() {
    let mut bench = Bench::new();
    let vessel = &mut bench.vessels[0];
    vessel.deposit_lot(
        SpeciesId::new("water"),
        Moles(5.5343),
        Phase::Liquid,
        Some("test medium".to_string()),
        None,
    );
    vessel.deposit_lot(
        SpeciesId::new("AgCl"),
        Moles(0.007),
        Phase::Solid,
        Some("test suspension".to_string()),
        Some(1.0),
    );

    let operator = parse_op("centrifuge v1 3000rpm 60s 8cm").unwrap().unwrap();
    let events = bench.step(operator).unwrap();
    let Event::Centrifuged {
        rcf,
        separations,
        state_coupled,
        ..
    } = events
        .iter()
        .find(|event| matches!(event, Event::Centrifuged { .. }))
        .unwrap()
    else {
        unreachable!()
    };
    assert!((*rcf - 805.1).abs() < 0.2);
    assert_eq!(separations.len(), 1);
    assert_eq!(separations[0].particle_diameter_um, 1.0);
    assert!(separations[0].separated_fraction > 0.0);
    assert!(!state_coupled);
}

#[test]
fn centrifuge_refuses_a_dry_solid() {
    let mut bench = Bench::new();
    bench.vessels[0].deposit_lot(
        SpeciesId::new("AgCl"),
        Moles(0.007),
        Phase::Solid,
        None,
        Some(1.0),
    );
    let error = bench
        .step(parse_op("centrifuge v1 3000rpm 60s 8cm").unwrap().unwrap())
        .unwrap_err();
    assert!(matches!(error, BenchError::CentrifugeUnavailable(_)));
}
