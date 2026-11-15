//! Synthetic admitted-activity readout controls, not a native RX012 qualification.
use kerotakis_core::{
    displacement, ops::CellTerminalReading, render::render_event_in, Bench, Event, Locale, Moles,
    Operator, Phase, Register, SolutionInfo, SpeciesDetail, SpeciesId, Vessel, VesselId,
};

fn half_cell(id: usize, activity: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(id), "polarity control");
    v.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    v.deposit(SpeciesId::new("Zn"), Moles(0.01), Phase::Solid);
    v.deposit(SpeciesId::new("Zn+2"), Moles(0.001), Phase::Aqueous);
    v.solution = Some(SolutionInfo {
        scope: Default::default(),
        solvent_kg: None,
        particle_molality: None,
        ph: 7.0,
        pe: None,
        redox: vec![],
        ionic_strength: 0.01,
        species: vec![SpeciesDetail {
            name: "Zn+2".into(),
            molality: 0.01,
            activity,
        }],
        provenance: None,
        solvent_activity: None,
    });
    v
}

fn reading(events: &[Event]) -> (&CellTerminalReading, VesselId, f64) {
    events
        .iter()
        .find_map(|event| match event {
            Event::CellVoltage {
                terminals: Some(reading),
                anode,
                volts,
                ..
            } => Some((reading, *anode, *volts)),
            _ => None,
        })
        .expect("ordered terminal readout")
}

#[test]
fn reverse_leads_reverse_signed_readout_without_reversing_chemistry_or_changing_stock() {
    let mut bench = Bench::new();
    bench.vessels = vec![half_cell(0, 1e-4), half_cell(1, 1e-2)];
    let before = serde_json::to_value(&bench.vessels).unwrap();
    let ea = displacement::electrode(&bench.vessels[0]).unwrap().volts;
    let eb = displacement::electrode(&bench.vessels[1]).unwrap().volts;
    let forward = bench
        .step(Operator::Cell {
            a: VesselId(0),
            b: VesselId(1),
        })
        .unwrap();
    let reverse = bench
        .step(Operator::Cell {
            a: VesselId(1),
            b: VesselId(0),
        })
        .unwrap();
    let (f, fa, fm) = reading(&forward);
    let (r, ra, rm) = reading(&reverse);
    assert_eq!((f.first, f.second), (VesselId(0), VesselId(1)));
    assert_eq!((r.first, r.second), (VesselId(1), VesselId(0)));
    assert!(f.volts > 0.0 && r.volts < 0.0);
    assert_eq!(f.volts, eb - ea);
    assert_eq!(r.volts, ea - eb);
    assert_eq!(fa, ra, "lead order does not change the spontaneous anode");
    assert_eq!(fm, rm, "legacy spontaneous voltage stays positive");
    assert_eq!(before, serde_json::to_value(&bench.vessels).unwrap());
}

#[test]
fn unresolved_or_equal_activity_does_not_gain_polarity_authority() {
    let a = half_cell(0, 1e-4);
    let mut b = half_cell(1, 1e-2);
    b.solution = None;
    assert!(displacement::cell(&a, &b).is_err());
    for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let b = half_cell(1, invalid);
        assert!(displacement::cell(&a, &b).is_err());
    }
    assert!(displacement::cell(&a, &half_cell(1, 1e-4)).is_err());
}

#[test]
fn legacy_event_has_no_invented_lead_order_and_roundtrips_unchanged() {
    let value = serde_json::json!({"event":"cell_voltage","anode":0,"cathode":1,
        "volts":0.04,"standard_volts":0.0,"notation":"Zn cell","equation":"Zn exchange"});
    let event: Event = serde_json::from_value(value.clone()).unwrap();
    assert!(matches!(
        event,
        Event::CellVoltage {
            terminals: None,
            ..
        }
    ));
    assert_eq!(serde_json::to_value(&event).unwrap(), value);
    assert!(!render_event_in(&event, Register::LV3, Locale::EN).contains("Ordered voltmeter"));
}

#[test]
fn ordered_negative_reading_is_visible_in_each_locale() {
    let event = Event::CellVoltage {
        anode: VesselId(0),
        cathode: VesselId(1),
        volts: 0.04,
        standard_volts: 0.0,
        notation: "Zn cell".into(),
        equation: "Zn exchange".into(),
        terminals: Some(CellTerminalReading {
            first: VesselId(1),
            second: VesselId(0),
            volts: -0.04,
        }),
    };
    for language in ["en", "de", "fr"] {
        let locale = Locale::parse(language);
        let line = render_event_in(&event, Register::LV3, locale);
        assert!(
            line.contains(&locale.number("-0.040000".to_string())),
            "{line}"
        );
        assert!(
            !line.contains("{first}") && !line.contains("{second}") && !line.contains("{volts}")
        );
    }
}
