//! Causal sample-admission controls; no equilibrium or calibration claim.
use kerotakis_core::instrument::Spectrophotometer;
use kerotakis_core::*;

fn water() -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "sample");
    vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    vessel
}

fn measure(vessel: Vessel) -> (Vec<Event>, Vessel) {
    let mut bench = Bench::new();
    bench.vessels[0] = vessel;
    let events = bench
        .step_with(
            Operator::Measure {
                vessel: VesselId(0),
                instrument: Instrument::Spectrophotometer,
            },
            &mut SolverStack::new(vec![]),
            &PermissiveScreen,
        )
        .unwrap();
    (events, bench.vessels.remove(0))
}

#[test]
fn liquid_blank_is_applicable_and_reads_zero() {
    let vessel = water();
    let instrument = Spectrophotometer::default();
    assert!(instrument.applies(&vessel));
    assert_eq!(instrument.measure(&vessel).unwrap().value, 0.0);
}

#[test]
fn empty_dry_and_gas_samples_have_no_reading() {
    let instrument = Spectrophotometer::default();
    let empty = Vessel::new(VesselId(0), "empty");
    assert!(!instrument.applies(&empty));
    assert!(instrument.measure(&empty).is_none());
    for phase in [Phase::Solid, Phase::Gas] {
        let mut vessel = Vessel::new(VesselId(0), "dry");
        vessel.deposit(SpeciesId::new("water"), Moles(1.0), phase);
        assert!(!instrument.applies(&vessel));
        assert!(instrument.measure(&vessel).is_none());
    }
}

#[test]
fn nonfinite_or_nonpositive_portions_are_not_samples() {
    let instrument = Spectrophotometer::default();
    for phase in [Phase::Aqueous, Phase::Liquid] {
        for amount in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut vessel = Vessel::new(VesselId(0), "invalid");
            vessel.deposit(SpeciesId::new("water"), Moles(1.0), phase);
            vessel.contents[0].moles = Moles(amount);
            assert!(!instrument.applies(&vessel));
            assert!(instrument.measure(&vessel).is_none());
        }
    }
}

#[test]
fn bench_without_liquid_refuses_locally_without_mutation() {
    for phase in [None, Some(Phase::Solid), Some(Phase::Gas)] {
        let mut vessel = Vessel::new(VesselId(0), "no liquid");
        if let Some(phase) = phase {
            vessel.deposit(SpeciesId::new("water"), Moles(1.0), phase);
        }
        let before = serde_json::to_value(&vessel).unwrap();
        let (events, after) = measure(vessel);
        assert_eq!(serde_json::to_value(after).unwrap(), before);
        assert!(!events
            .iter()
            .any(|event| matches!(event, Event::Measured { .. })));
        let reason = events
            .iter()
            .find_map(|event| match event {
                Event::NotYetModeled {
                    reason: Some(reason),
                    ..
                } if reason.key == "not-modeled.spectrophotometer-has-no-solution" => Some(reason),
                _ => None,
            })
            .expect("explicit existing localized no-sample refusal");
        for locale in [Locale::parse("de"), Locale::parse("fr")] {
            assert_ne!(reason.render(locale), reason.render(Locale::EN));
        }
    }
}

#[test]
fn bench_pure_water_blank_is_measured_without_mutation() {
    let vessel = water();
    let before = serde_json::to_value(&vessel).unwrap();
    let (events, after) = measure(vessel);
    assert_eq!(serde_json::to_value(after).unwrap(), before);
    assert!(events.iter().any(|event| matches!(event,
        Event::Measured { instrument: Instrument::Spectrophotometer, value, .. }
        if *value == 0.0)));
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::NotYetModeled { .. })));
}

#[test]
fn aqueous_supported_absorber_still_measures() {
    let mut vessel = water();
    vessel.deposit(SpeciesId::new("MnO4-"), Moles(1e-6), Phase::Aqueous);
    let instrument = Spectrophotometer::default();
    assert!(instrument.applies(&vessel));
    assert!(instrument.measure(&vessel).unwrap().value > 0.0);
    let before = serde_json::to_value(&vessel).unwrap();
    let (events, after) = measure(vessel);
    assert_eq!(serde_json::to_value(after).unwrap(), before);
    assert!(events.iter().any(|event| matches!(event,
        Event::Measured { instrument: Instrument::Spectrophotometer, value, .. }
        if *value > 0.0)));
}
