use kerotakis_core::{i18n::Locale, render::render_event_in};
use kerotakis_core::{script::parse_op, Event, Operator, Register, VesselId};

#[test]
fn signed_lead_syntax_is_explicit_and_legacy_alias_stays_unchanged() {
    let op = parse_op("voltmeter positive v2 negative v1")
        .unwrap()
        .unwrap();
    assert_eq!(
        op,
        Operator::Voltmeter {
            positive: VesselId(1),
            negative: VesselId(0)
        }
    );
    let json = serde_json::to_string(&op).unwrap();
    assert_eq!(op, serde_json::from_str::<Operator>(&json).unwrap());
    assert_eq!(
        parse_op("voltmeter v1 v2").unwrap(),
        parse_op("cell v1 v2").unwrap()
    );
    for bad in [
        "voltmeter positive v1 negative",
        "voltmeter positive v1 negative v2 ignored",
        "voltmeter negative v1 positive v2",
        "voltmeter v1 v2 ignored",
        "cell positive v1 negative v2",
    ] {
        assert!(parse_op(bad).is_err(), "accepted {bad}");
    }
}

#[test]
fn signed_readout_names_both_leads_and_keeps_the_sign_in_each_locale() {
    for volts in [-0.0532, 0.0532] {
        let event = Event::VoltmeterRead {
            positive: VesselId(0),
            negative: VesselId(1),
            volts,
        };
        for register in [Register::LV1, Register::LV2, Register::LV3] {
            for (lang, number, positive, negative) in [
                ("en", "0.0532", "positive lead v1", "negative lead v2"),
                ("de", "0,0532", "Plusleitung v1", "Minusleitung v2"),
                ("fr", "0,0532", "borne positive v1", "borne négative v2"),
            ] {
                let rendered = render_event_in(&event, register, Locale::parse(lang));
                assert!(
                    rendered.contains(&format!("{}{number}", if volts < 0.0 { "-" } else { "+" })),
                    "{rendered}"
                );
                assert!(
                    rendered.contains(positive) && rendered.contains(negative),
                    "{rendered}"
                );
            }
        }
    }
}

fn acid_metal(id: usize, metal: &str, ph: f64) -> kerotakis_core::Vessel {
    use kerotakis_core::{Moles, Phase, SpeciesId, Vessel};
    let mut vessel = Vessel::new(VesselId(id), "acid electrode");
    vessel.deposit(SpeciesId::new("water"), Moles(2.75), Phase::Liquid);
    vessel.deposit(SpeciesId::new(metal), Moles(0.01), Phase::Solid);
    vessel.solution = Some(kerotakis_core::vessel::SolutionInfo {
        solvent_activity: None,
        scope: Default::default(),
        solvent_kg: None,
        particle_molality: None,
        ph,
        pe: None,
        redox: Vec::new(),
        ionic_strength: 0.02,
        species: Vec::new(),
        provenance: None,
    });
    vessel
}

#[test]
fn signed_acid_estimate_reverses_and_retains_its_bounded_explanation() {
    use kerotakis_core::Bench;
    let mut bench = Bench::new();
    bench.vessels = vec![acid_metal(0, "Zn", 1.86), acid_metal(1, "Cu", 1.86)];
    let before = serde_json::to_string(&bench.vessels).unwrap();
    let mut readings = Vec::new();
    for (positive, negative) in [(0, 1), (1, 0)] {
        let events = bench
            .step(Operator::Voltmeter {
                positive: VesselId(positive),
                negative: VesselId(negative),
            })
            .unwrap();
        let signed = events
            .iter()
            .find_map(|e| match e {
                Event::VoltmeterRead { volts, .. } => Some(*volts),
                _ => None,
            })
            .unwrap();
        assert!(events.iter().any(
            |e| matches!(e, Event::AcidMetalCellVoltage { volts, .. } if *volts == signed.abs())
        ));
        readings.push(signed);
    }
    assert!(readings[0] < 0.0);
    assert_eq!(readings[0], -readings[1]);
    assert_eq!(before, serde_json::to_string(&bench.vessels).unwrap());
    for ph in [5.0, 7.0] {
        bench.vessels = vec![acid_metal(0, "Zn", ph), acid_metal(1, "Cu", ph)];
        let events = bench
            .step(Operator::Voltmeter {
                positive: VesselId(0),
                negative: VesselId(1),
            })
            .unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::NoCell { .. })));
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::VoltmeterRead { .. })));
    }
}
