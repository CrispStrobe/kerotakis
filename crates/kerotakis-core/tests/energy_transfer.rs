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
            ..
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

#[test]
fn delivered_energy_is_rendered_by_the_engine_in_german() {
    use kerotakis_core::render::{render_event_in, Register};

    let event = Event::EnergyTransferred {
        vessel: VesselId(0),
        heating: false,
        requested_j: 5000.0,
        delivered_j: 2500.0,
        time_coupled: false,
        source: None,
        ceiling_k: None,
        sensible_j: 2500.0,
        energy_partition_complete: None,
        passes: 1,
        capped: false,
    };
    let line = render_event_in(&event, Register::LV2, Locale::parse("de"));
    assert!(line.contains("5,00 kJ angefordert"), "{line}");
    assert!(line.contains("2,50 kJ entzogen"), "{line}");
    assert!(line.contains("noch nicht gekoppelt"), "{line}");
    assert!(!line.contains("requested"), "{line}");
}

#[test]
fn incomplete_partition_is_explicit_in_every_register_and_saved_events_still_load() {
    let mut event = Event::EnergyTransferred {
        vessel: VesselId(0), heating: true, requested_j: 5000.0,
        delivered_j: 5000.0, time_coupled: false, source: None,
        ceiling_k: None, sensible_j: 1000.0,
        energy_partition_complete: Some(false), passes: 1, capped: false,
    };
    for register in [render::Register::LV1, render::Register::LV2, render::Register::LV3] {
        let text = render::render_event(&event, register);
        assert!(text.contains("split") && text.contains("not fully established"), "{text}");
        assert!(!text.contains("chemistry/phase="), "an uncertified residual is not an exact phase budget: {text}");
    }
    let mut old = serde_json::to_value(&event).unwrap();
    old.as_object_mut().unwrap().remove("energy_partition_complete");
    event = serde_json::from_value(old).unwrap();
    assert!(matches!(event, Event::EnergyTransferred { energy_partition_complete: None, .. }));
}
