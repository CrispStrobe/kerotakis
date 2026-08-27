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

#[test]
fn irradiation_evidence_is_rendered_by_the_engine_in_german() {
    use kerotakis_core::render::{render_event_in, Register};

    let event = Event::Irradiated {
        vessel: VesselId(0),
        wavelength_nm: 254.0,
        irradiance_w_m2: 12.5,
        photolysis_coupled: false,
    };
    let line = render_event_in(&event, Register::LV2, Locale::parse("de"));
    assert!(line.contains("Lampe 254 nm"), "{line}");
    assert!(line.contains("12,50 W/m²"), "{line}");
    assert!(line.contains("noch nicht gekoppelt"), "{line}");
    assert!(!line.contains("not yet coupled"), "{line}");
}
