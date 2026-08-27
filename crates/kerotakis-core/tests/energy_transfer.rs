use kerotakis_core::script::parse_op;
use kerotakis_core::*;

#[test]
fn heat_reports_the_energy_actually_delivered_and_its_time_boundary() {
    let mut bench = Bench::new();
    bench
        .step(parse_op("add v1 water 100mL").unwrap().unwrap())
        .unwrap();
    let events = bench
        .step(parse_op("heat v1 1kJ").unwrap().unwrap())
        .unwrap();

    assert!(events.iter().any(|event| matches!(
        event,
        Event::EnergyTransferred {
            vessel: VesselId(0),
            heating: true,
            requested_j,
            delivered_j,
            time_coupled: false,
        } if (*requested_j - 1000.0).abs() < 1e-9
            && (*delivered_j - 1000.0).abs() < 1e-6
    )));
}

#[test]
fn impossible_cooling_reports_less_delivered_than_requested() {
    let mut bench = Bench::new();
    bench
        .step(parse_op("add v1 water 100mL").unwrap().unwrap())
        .unwrap();
    let events = bench
        .step(parse_op("cool v1 1000kJ").unwrap().unwrap())
        .unwrap();

    assert!(events.iter().any(|event| matches!(
        event,
        Event::EnergyTransferred {
            heating: false,
            requested_j,
            delivered_j,
            time_coupled: false,
            ..
        } if *delivered_j < *requested_j
    )));
}
