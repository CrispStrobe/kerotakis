//! Numerical refusal is distinct from missing molecular data at transfer seams.
use kerotakis_core::*;
fn op(s: &str) -> Operator {
    script::parse_op(s).unwrap().unwrap()
}
fn sample(layered: bool) -> Bench {
    let mut b = Bench::new();
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    b.vessels[0].deposit(SpeciesId::new("ethanol"), Moles(0.01), Phase::Aqueous);
    if layered {
        b.vessels[0].deposit(SpeciesId::new("hexane"), Moles(0.5), Phase::Liquid);
    }
    b
}
fn numerical_notice(events: &[Event]) {
    let phrase = events
        .iter()
        .find_map(|event| match event {
            Event::NotYetModeled {
                reason: Some(p), ..
            } if p.key == "not-modeled.liquid-partition-refused" => Some(p),
            _ => None,
        })
        .expect("retain numerical refusal, distinct from missing distribution data");
    for (locale, word) in [
        (Locale::parse("de"), "Verteilungskoeffizienten"),
        (Locale::parse("fr"), "coefficient de partage"),
    ] {
        let text = phrase.render(locale);
        assert!(text.contains(word), "{text}");
        assert!(!text.contains("not a finite positive temperature"));
        assert_ne!(text, phrase.render(Locale::EN));
    }
}
#[test]
fn refused_drain_creates_no_receiver_and_moves_no_material() {
    for temperature in [0.0, 1e-300] {
        let mut b = sample(true);
        b.vessels[0].temperature = Kelvin(temperature);
        let before = serde_json::to_value(&b.vessels).unwrap();
        let events = b.step(op("drain v1 v2")).unwrap();
        assert_eq!(
            serde_json::to_value(&b.vessels).unwrap(),
            before,
            "atomic refusal includes destination creation"
        );
        numerical_notice(&events);
    }
}
#[test]
fn refused_extraction_preserves_the_numerical_reason_and_inventory() {
    for temperature in [0.0, 1e-300] {
        let mut b = sample(false);
        b.vessels[0].temperature = Kelvin(temperature);
        let before = serde_json::to_value(&b.vessels).unwrap();
        let events = b.step(op("extract v1 v2 hexane 0.5mol stages 2")).unwrap();
        assert_eq!(serde_json::to_value(&b.vessels).unwrap(), before);
        numerical_notice(&events);
        assert!(!events.iter().any(|e| matches!(e, Event::Extracted { .. })));
    }
}
#[test]
fn ordinary_supported_drain_retains_both_solute_shares_and_total() {
    let mut b = sample(true);
    b.step(op("drain v1 v2")).unwrap();
    assert_eq!(b.vessels.len(), 2);
    let ethanol = SpeciesId::new("ethanol");
    let shares: Vec<_> = b.vessels.iter().map(|v| v.moles_of(&ethanol).0).collect();
    assert!(shares.iter().all(|n| *n > 0.0 && *n < 0.01));
    assert!((shares.iter().sum::<f64>() - 0.01).abs() < 1e-12);
}
