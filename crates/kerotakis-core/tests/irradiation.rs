use kerotakis_core::script::parse_op;
use kerotakis_core::*;

#[test]
fn irradiation_reports_applied_light_without_claiming_photochemistry() {
    let mut bench = Bench::new();
    let operator = parse_op("irradiate v1 254nm 12.5W/m2").unwrap().unwrap();
    let events = bench.step(operator).unwrap();

    assert!(events.iter().any(|event| matches!(
        event,
        Event::Irradiated {
            vessel: VesselId(0),
            wavelength_nm,
            irradiance_w_m2,
            photolysis_coupled: false,
        } if (*wavelength_nm - 254.0).abs() < f64::EPSILON
            && (*irradiance_w_m2 - 12.5).abs() < f64::EPSILON
    )));
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::NotYetModeled { .. })));
}
