use kerotakis_core::{
    script::parse_op, Bench, Event, Moles, Phase, SafetyScreen, SafetyVerdict, Severity,
    SolverStack, SpeciesId, Vessel, VesselId,
};

struct ReceiverScreen;

impl SafetyScreen for ReceiverScreen {
    fn assess(&self, vessel: &Vessel) -> SafetyVerdict {
        let has = |key: &str| {
            vessel
                .contents
                .iter()
                .any(|portion| portion.species.0 == key && portion.moles.0 > 0.0)
        };
        if has("H2O2") && has("hexane") {
            SafetyVerdict::Warn {
                severity: Severity::Danger,
                rule: "oxidizer-flammable-liquid".to_string(),
                hazard: "test receiver hazard".to_string(),
                real_world: "test evidence".to_string(),
            }
        } else {
            SafetyVerdict::Allow
        }
    }
}

fn run(stages: u32) -> (Bench, Vec<Event>) {
    let mut bench = Bench::new();
    for line in ["new", "add v1 water 2mol", "add v1 I2 0.001mol"] {
        bench
            .step(parse_op(line).unwrap().expect("operator"))
            .unwrap();
    }
    let events = bench
        .step(
            parse_op(&format!("extract v1 v2 hexane 0.5mol stages {stages}"))
                .unwrap()
                .expect("extract operator"),
        )
        .unwrap();
    (bench, events)
}

#[test]
fn repeated_fresh_portions_extract_more_and_conserve_every_species() {
    let (single, single_events) = run(1);
    let (repeated, repeated_events) = run(4);
    let iodine = SpeciesId::new("I2");
    let hexane = SpeciesId::new("hexane");

    let extracted = |bench: &Bench| bench.vessel(VesselId(1)).unwrap().moles_of(&iodine).0;
    assert!(extracted(&repeated) > extracted(&single));
    for bench in [&single, &repeated] {
        let iodine_total = bench
            .vessels
            .iter()
            .map(|v| v.moles_of(&iodine).0)
            .sum::<f64>();
        let hexane_total = bench
            .vessels
            .iter()
            .map(|v| v.moles_of(&hexane).0)
            .sum::<f64>();
        assert!((iodine_total - 0.001).abs() < 1e-12);
        assert!((hexane_total - 0.5).abs() < 1e-12);
    }

    let split = repeated_events
        .iter()
        .find_map(|event| match event {
            Event::Extracted {
                stages, solutes, ..
            } if *stages == 4 => solutes.iter().find(|split| split.species == iodine),
            _ => None,
        })
        .expect("typed extraction result");
    assert_eq!(split.partition_k, 85.0);
    assert!(split.staged_efficiency > split.single_stage_efficiency);
    assert!((split.extracted.0 + split.remaining.0 - 0.001).abs() < 1e-12);
    assert!(split.provenance.contains("Edexcel"));

    assert!(single_events
        .iter()
        .any(|event| matches!(event, Event::Extracted { stages: 1, .. })));
    assert!(!repeated_events
        .iter()
        .any(|event| matches!(event, Event::HazardWarning { .. })));
}

#[test]
fn unsupported_solutes_and_out_of_range_temperature_are_explicit_and_atomic() {
    let mut bench = Bench::new();
    for line in ["new", "add v1 water 2mol", "add v1 NaCl 0.001mol"] {
        bench
            .step(parse_op(line).unwrap().expect("operator"))
            .unwrap();
    }
    let before = serde_json::to_value(&bench.vessels).unwrap();
    let events = bench
        .step(
            parse_op("extract v1 v2 hexane 0.5mol stages 2")
                .unwrap()
                .expect("operator"),
        )
        .unwrap();
    assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::NotYetModeled { what, .. } if what.contains("no reviewed hexane/water")
    )));

    let (mut iodine_bench, _) = run(1);
    iodine_bench
        .vessels
        .iter_mut()
        .find(|vessel| vessel.id == VesselId(0))
        .unwrap()
        .temperature
        .0 = 310.0;
    let before_iodine = serde_json::to_value(&iodine_bench.vessels).unwrap();
    let events = iodine_bench
        .step(
            parse_op("extract v1 v2 hexane 0.1mol")
                .unwrap()
                .expect("operator"),
        )
        .unwrap();
    assert_eq!(
        serde_json::to_value(&iodine_bench.vessels).unwrap(),
        before_iodine
    );
    assert!(events.iter().any(|event| matches!(
        event,
        Event::NotYetModeled { what, .. } if what.contains("only reviewed at")
    )));
}

#[test]
fn solid_iodine_outside_the_reviewed_solubility_domain_is_atomic() {
    let mut bench = Bench::new();
    for line in ["add v1 water 2mol", "add v1 I2 0.01mol"] {
        bench
            .step(parse_op(line).unwrap().expect("operator"))
            .unwrap();
    }
    let before = serde_json::to_value(&bench.vessels).unwrap();
    let events = bench
        .step(
            parse_op("extract v1 v2 hexane 0.5mol stages 4")
                .unwrap()
                .expect("operator"),
        )
        .unwrap();
    assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::NotYetModeled { what, .. } if what.contains("three-phase equilibrium")
    )));
}

#[test]
fn a_dissolved_supersaturated_state_is_not_mistaken_for_a_solid_reservoir() {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    bench.vessels[0].deposit(SpeciesId::new("I2"), Moles(0.01), Phase::Aqueous);

    let events = bench
        .step(
            parse_op("extract v1 v2 hexane 0.5mol")
                .unwrap()
                .expect("operator"),
        )
        .unwrap();
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::Extracted { .. })));
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::NotYetModeled { what, .. } if what.contains("three-phase equilibrium")
    )));
}

#[test]
fn group_model_extracts_a_dissolved_supported_solute_without_a_named_case() {
    let mut bench = Bench::new();
    bench
        .step(parse_op("add v1 water 2mol").unwrap().expect("operator"))
        .unwrap();
    bench.vessels[0].deposit(SpeciesId::new("ethanol"), Moles(0.001), Phase::Aqueous);

    let events = bench
        .step(
            parse_op("extract v1 v2 hexane 0.5mol stages 3")
                .unwrap()
                .expect("operator"),
        )
        .unwrap();
    let split = events
        .iter()
        .find_map(|event| match event {
            Event::Extracted { solutes, .. } => solutes
                .iter()
                .find(|split| split.species == SpeciesId::new("ethanol")),
            _ => None,
        })
        .expect("UNIFAC extraction result");
    assert!(split.partition_k.is_finite() && split.partition_k > 0.0);
    assert!(split.model.contains("UNIFAC"));
    assert!((split.extracted.0 + split.remaining.0 - 0.001).abs() < 1e-12);
}

#[test]
fn extraction_grammar_defaults_to_one_stage_and_rejects_nonfinite_or_zero_inputs() {
    assert!(matches!(
        parse_op("extract v1 v2 hexane 0.5mol").unwrap(),
        Some(kerotakis_core::Operator::Extract { stages: 1, .. })
    ));
    assert!(parse_op("extract v1 v2 hexane 0.5mol stages 0").is_err());

    let mut bench = Bench::new();
    bench
        .step(parse_op("add v1 water 2mol").unwrap().expect("operator"))
        .unwrap();
    let before = serde_json::to_value(&bench.vessels).unwrap();
    assert!(parse_op("extract v1 v2 hexane NaNmol").is_err());
    assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
}

#[test]
fn reviewed_iodine_solubility_feeds_the_standing_water_hexane_partition() {
    let mut bench = Bench::new();
    let mut iodine_add_events = Vec::new();
    for line in ["add v1 water 2mol", "add v1 hexane 0.5mol"] {
        bench
            .step(parse_op(line).unwrap().expect("operator"))
            .unwrap();
    }
    iodine_add_events.extend(
        bench
            .step(parse_op("add v1 I2 0.001mol").unwrap().expect("operator"))
            .unwrap(),
    );
    assert!(iodine_add_events.iter().any(|event| matches!(
        event,
        Event::Dissolved { species, moles, .. } if species.0 == "I2" && moles.0 > 0.0
    )));
    assert!(!iodine_add_events.iter().any(|event| matches!(
        event,
        Event::NotYetModeled { what, .. } if what.contains("iodine in contact")
    )));

    let scene = kerotakis_core::scene(&bench);
    let standing = scene.vessels[0]
        .partition
        .iter()
        .find(|split| split.species == "I2")
        .expect("iodine standing partition");
    assert!(standing.upper_moles > standing.lower_moles);
    assert!((standing.lower_moles + standing.upper_moles - standing.total_moles).abs() < 1e-12);
    assert!(standing.provenance.contains("Edexcel"));
}

#[test]
fn draining_outside_an_empirical_partition_temperature_is_atomic() {
    let mut bench = Bench::new();
    for line in [
        "add v1 water 2mol",
        "add v1 hexane 0.5mol",
        "add v1 I2 0.001mol",
    ] {
        bench
            .step(parse_op(line).unwrap().expect("operator"))
            .unwrap();
    }
    bench.vessels[0].temperature.0 = 310.0;
    let before = serde_json::to_value(&bench.vessels).unwrap();
    let events = bench
        .step(parse_op("drain v1 v2").unwrap().expect("operator"))
        .unwrap();
    assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::NotYetModeled { what, .. } if what.contains("only reviewed at")
    )));
}

#[test]
fn extraction_refuses_an_existing_second_layer_and_screens_the_receiver() {
    let mut layered = Bench::new();
    for line in [
        "add v1 water 2mol",
        "add v1 hexane 0.1mol",
        "add v1 I2 0.0001mol",
    ] {
        layered
            .step(parse_op(line).unwrap().expect("operator"))
            .unwrap();
    }
    let before = serde_json::to_value(&layered.vessels).unwrap();
    let events = layered
        .step(
            parse_op("extract v1 v2 hexane 0.1mol")
                .unwrap()
                .expect("operator"),
        )
        .unwrap();
    assert_eq!(serde_json::to_value(&layered.vessels).unwrap(), before);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::NotYetModeled { what, .. } if what.contains("existing organic layer")
    )));

    let mut receiver = Bench::new();
    for line in [
        "add v1 water 2mol",
        "add v1 I2 0.0001mol",
        "new",
        "add v2 H2O2 0.01mol",
    ] {
        receiver
            .step(parse_op(line).unwrap().expect("operator"))
            .unwrap();
    }
    let mut solver = SolverStack::new(Vec::new());
    let events = receiver
        .step_with(
            parse_op("extract v1 v2 hexane 0.1mol")
                .unwrap()
                .expect("operator"),
            &mut solver,
            &ReceiverScreen,
        )
        .unwrap();
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::HazardWarning { rule, .. } if rule == "oxidizer-flammable-liquid")));
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::Extracted { .. })));
}
