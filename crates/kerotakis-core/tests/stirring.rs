use kerotakis_core::script::parse_op;
use kerotakis_core::*;

#[test]
fn stir_command_carries_physical_conditions_and_tip_speed() {
    let op = parse_op("stir v1 600rpm 30s").unwrap().unwrap();
    assert_eq!(
        op,
        Operator::Stir {
            vessel: VesselId(0),
            rpm: 600.0,
            seconds: 30.0,
        }
    );

    let mut bench = Bench::new();
    let events = bench.step(op).unwrap();
    let event = events
        .iter()
        .find_map(|event| match event {
            Event::Stirred {
                tip_speed_m_s,
                resuspended_fraction,
                rate_coupled,
                ..
            } => Some((*tip_speed_m_s, *resuspended_fraction, *rate_coupled)),
            _ => None,
        })
        .expect("stirred event");
    assert!((event.0 - std::f64::consts::PI * 0.025 * 10.0).abs() < 1e-12);
    assert!(event.1 > 0.99);
    assert!(!event.2, "rate coupling must remain explicitly bounded");
    assert!(render_events(&events, Register::LV2)[0].contains("600 rpm"));
}

#[test]
fn legacy_short_form_gets_a_bench_scale_default() {
    assert_eq!(
        parse_op("stir v1").unwrap().unwrap(),
        Operator::Stir {
            vessel: VesselId(0),
            rpm: 500.0,
            seconds: 10.0
        }
    );
}
