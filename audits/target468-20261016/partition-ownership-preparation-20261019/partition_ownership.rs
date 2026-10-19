//! Proposed ownership controls: not executed or accepted at preparation time.
use kerotakis_core::*;

fn op(command: &str) -> Operator {
    script::parse_op(command).unwrap().unwrap()
}

fn sample(layered: bool) -> Bench {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    if layered {
        bench.vessels[0].deposit(SpeciesId::new("hexane"), Moles(0.5), Phase::Liquid);
    }
    bench
}

#[test]
fn unparameterized_neutral_drain_refuses_without_assigning_a_layer() {
    for phase in [Phase::Liquid, Phase::Aqueous] {
        let mut bench = sample(true);
        bench.vessels[0].deposit(SpeciesId::new("ethyl_acetate"), Moles(0.01), phase);
        let before = serde_json::to_value(&bench.vessels).unwrap();
        let events = bench.step(op("drain v1 v2")).unwrap();
        assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
        assert!(!events.iter().any(|e| matches!(e, Event::Drained { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })));
    }
}

#[test]
fn supported_liquid_and_aqueous_solute_tags_give_the_same_extraction() {
    let mut shares = Vec::new();
    for phase in [Phase::Liquid, Phase::Aqueous] {
        let mut bench = sample(false);
        let ethanol = SpeciesId::new("ethanol");
        bench.vessels[0].deposit(ethanol.clone(), Moles(0.01), phase);
        let events = bench
            .step(op("extract v1 v2 hexane 0.5mol stages 2"))
            .unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::Extracted { .. })));
        assert_eq!(bench.vessels.len(), 2);
        let retained = bench.vessels[0].moles_of(&ethanol).0;
        let extracted = bench.vessels[1].moles_of(&ethanol).0;
        assert!(retained > 0.0 && extracted > 0.0);
        assert!((retained + extracted - 0.01).abs() < 1e-12);
        shares.push((retained, extracted));
    }
    assert!((shares[0].0 - shares[1].0).abs() < 1e-12);
    assert!((shares[0].1 - shares[1].1).abs() < 1e-12);
}

#[test]
fn explicit_dissolved_ions_follow_water_without_neutral_partition_data() {
    let mut bench = sample(true);
    let sodium = SpeciesId::new("Na+");
    let chloride = SpeciesId::new("Cl-");
    for ion in [&sodium, &chloride] {
        bench.vessels[0].deposit(ion.clone(), Moles(0.01), Phase::Aqueous);
    }
    let events = bench.step(op("drain v1 v2")).unwrap();
    assert!(events.iter().any(|e| matches!(e, Event::Drained { .. })));
    assert_eq!(bench.vessels.len(), 2);
    for ion in [&sodium, &chloride] {
        assert!(bench.vessels[0].moles_of(ion).0.abs() < 1e-12);
        assert!((bench.vessels[1].moles_of(ion).0 - 0.01).abs() < 1e-12);
    }
    assert!((bench.vessels[0].moles_of(&SpeciesId::new("hexane")).0 - 0.5).abs() < 1e-12);
    assert!((bench.vessels[1].moles_of(&SpeciesId::new("water")).0 - 2.0).abs() < 1e-12);
}
