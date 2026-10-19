//! A failed stability calculation supplies no phase verdict or mutation.
use kerotakis_core::*;
#[test]
fn checked_layer_detection_and_mixing_preserve_refused_state() {
    for temperature in [0.0, 1e-300] {
        let mut v = Vessel::new(VesselId(0), "layer control");
        v.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
        v.deposit(SpeciesId::new("hexane"), Moles(0.5), Phase::Liquid);
        v.temperature = Kelvin(temperature);
        let before = serde_json::to_value(&v).unwrap();
        assert!(solve::try_layered_pair(&v).is_err());
        assert!(solve::layered_pair(&v).is_none());
        let events = MixingEquilibrator.equilibrate(&mut v).unwrap();
        assert_eq!(serde_json::to_value(&v).unwrap(), before);
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::LayersFormed { .. })));
        assert!(events.iter().any(|e|matches!(e,Event::NotYetModeled { reason:Some(p),.. } if p.key=="not-modeled.liquid-partition-refused")));
    }
}
