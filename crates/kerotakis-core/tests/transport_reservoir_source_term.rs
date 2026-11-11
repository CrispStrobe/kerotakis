//! The constant-reservoir inlet is a stated, closable source term.
//!
//! Review of batch3-diverse:tr-006 summed every vessel after two transport
//! commands and found the dye, Na and Cl doubled and 4.43 mol of extra
//! water: the default inlet is an external constant-composition reservoir
//! (as a PHREEQC TRANSPORT boundary solution) that is never debited, and
//! nothing a reader saw said so. The event already carried `inflow_moles`;
//! these tests pin the reader-facing contract built on it:
//!
//! * every transport event names its inlet and its boundary mode, and the
//!   rendered line states the reservoir inflow per species at every
//!   register in EN/DE/FR;
//! * for the tr-006 shape (two reservoirs feeding one charcoal column in
//!   turn) the ledger closes per species counting the bound surface pool:
//!   cells + receiver + reservoirs + adsorbed - sum(inflow) == initial;
//! * a drained (finite) feed closes with no source term at all.

use kerotakis_core::adsorption::AdsorptionEquilibrator;
use kerotakis_core::i18n::Locale;
use kerotakis_core::ops::{Event, Operator};
use kerotakis_core::render::{render_event_in, Register};
use kerotakis_core::species::{self, Phase, SpeciesId};
use kerotakis_core::units::{Kelvin, Moles};
use kerotakis_core::vessel::{Vessel, VesselId};
use kerotakis_core::{Bench, PermissiveScreen};

const DYE: &str = "methyl_orange";
const CARBON: &str = "activated_charcoal";
/// `add vN water 20mL` (the tr-006 production event).
const WATER_20ML: f64 = 1.1068553982792118;
/// `add v4 methyl_orange 0.001g` (the tr-006 production event).
const DYE_MOLES: f64 = 3.0549835947380964e-6;
const SPECIES: [&str; 4] = ["water", DYE, "NaCl", CARBON];

fn water(vessel: &mut Vessel, volumes: f64) {
    vessel.deposit(
        SpeciesId::new("water"),
        Moles(WATER_20ML * volumes),
        Phase::Liquid,
    );
    vessel.temperature = Kelvin::STANDARD;
}

/// The tr-006 bench: v1 charcoal cell, v2/v3 water cells, v4 dye + salt
/// inlet, v5 empty receiver, v6 clean-water inlet. `second_inlet_volumes`
/// is 1 for the original; a drained run needs three cell volumes there.
fn tr006(second_inlet_volumes: f64) -> Bench {
    let mut bench = Bench::new();
    for _ in 0..5 {
        bench.step(Operator::NewVessel { kind: None }).unwrap();
    }
    for index in 0..4 {
        water(&mut bench.vessels[index], 1.0);
    }
    let carbon = species::lookup(&SpeciesId::new(CARBON)).expect("registry carbon");
    bench.vessels[0].deposit(
        SpeciesId::new(CARBON),
        Moles(0.02 / carbon.molar_mass),
        Phase::Solid,
    );
    bench.vessels[3].deposit(SpeciesId::new(DYE), Moles(DYE_MOLES), Phase::Aqueous);
    bench.vessels[3].deposit(SpeciesId::new("NaCl"), Moles(1e-4), Phase::Aqueous);
    water(&mut bench.vessels[5], second_inlet_volumes);
    bench
}

fn transport(inlet: usize, steps: u32, drain: bool) -> Operator {
    Operator::Transport {
        chain: vec![VesselId(0), VesselId(1), VesselId(2)],
        inlet: VesselId(inlet),
        receiver: VesselId(4),
        steps,
        courant: 0.5,
        drain,
    }
}

/// Dissolved, solid and surface-bound amount of a species on the bench.
fn everywhere(bench: &Bench, species: &str) -> f64 {
    let id = SpeciesId::new(species);
    bench
        .vessels
        .iter()
        .map(|v| v.moles_of(&id).0 + v.adsorbed_moles(&id).0)
        .sum()
}

fn transported(events: &[Event]) -> &Event {
    events
        .iter()
        .find(|e| matches!(e, Event::Transported { .. }))
        .expect("a transport event")
}

fn inflow(event: &Event, species: &str) -> f64 {
    let Event::Transported { inflow_moles, .. } = event else {
        unreachable!()
    };
    let id = SpeciesId::new(species);
    inflow_moles
        .iter()
        .filter(|(s, _)| *s == id)
        .map(|(_, m)| m.0)
        .sum()
}

fn close(actual: f64, expected: f64, what: &str) {
    let scale = expected.abs().max(1e-30);
    assert!(
        (actual - expected).abs() <= 1e-12 * scale,
        "{what}: {actual:e} vs {expected:e}"
    );
}

#[test]
fn tr006_two_reservoir_ledger_closes_with_the_surface_pool() {
    let mut bench = tr006(1.0);
    let initial: Vec<f64> = SPECIES.iter().map(|s| everywhere(&bench, s)).collect();
    let reservoirs = [bench.vessels[3].clone(), bench.vessels[5].clone()];

    let mut solver = AdsorptionEquilibrator;
    let first = bench
        .step_with(transport(3, 2, false), &mut solver, &PermissiveScreen)
        .unwrap();
    let second = bench
        .step_with(transport(5, 6, false), &mut solver, &PermissiveScreen)
        .unwrap();
    let (first, second) = (transported(&first), transported(&second));

    for (event, inlet) in [(first, 3), (second, 5)] {
        let Event::Transported {
            inlet: named,
            drain,
            ..
        } = event
        else {
            unreachable!()
        };
        assert_eq!(*named, Some(VesselId(inlet)), "the event names its inlet");
        assert!(!drain, "the default boundary is a reservoir");
    }
    // Constant reservoirs: neither inlet was debited.
    for (index, before) in [(3, &reservoirs[0]), (5, &reservoirs[1])] {
        for species in SPECIES {
            let id = SpeciesId::new(species);
            assert_eq!(
                bench.vessels[index].moles_of(&id).0,
                before.moles_of(&id).0,
                "reservoir v{} {species}",
                index + 1
            );
        }
    }
    // The bed holds dye, so the surface pool is a real term of the sum.
    assert!(bench.vessels[0].adsorbed_moles(&SpeciesId::new(DYE)).0 > 0.0);
    // The duplication the reviewer saw is exactly the stated source term.
    assert!(inflow(first, DYE) > 0.0 && inflow(first, "NaCl") > 0.0);
    assert!(inflow(second, "water") > 3.0 * WATER_20ML * 0.99);
    for (species, initial) in SPECIES.iter().zip(&initial) {
        let source = inflow(first, species) + inflow(second, species);
        close(
            everywhere(&bench, species) - source,
            *initial,
            &format!("{species}: cells + receiver + reservoirs + surface - inflow"),
        );
    }
}

#[test]
fn tr006_drained_feeds_close_without_a_source_term() {
    let mut bench = tr006(3.0);
    let initial: Vec<f64> = SPECIES.iter().map(|s| everywhere(&bench, s)).collect();
    let feeds = [bench.vessels[3].clone(), bench.vessels[5].clone()];
    let mut solver = AdsorptionEquilibrator;
    let first = bench
        .step_with(transport(3, 2, true), &mut solver, &PermissiveScreen)
        .unwrap();
    let second = bench
        .step_with(transport(5, 6, true), &mut solver, &PermissiveScreen)
        .unwrap();
    for (species, initial) in SPECIES.iter().zip(&initial) {
        close(everywhere(&bench, species), *initial, species);
    }
    // What each event reports entering is what its feed lost.
    for (events, index, feed) in [(&first, 3, &feeds[0]), (&second, 5, &feeds[1])] {
        let event = transported(events);
        assert!(matches!(event, Event::Transported { drain: true, .. }));
        for species in ["water", DYE, "NaCl"] {
            let id = SpeciesId::new(species);
            let lost = feed.moles_of(&id).0 - bench.vessels[index].moles_of(&id).0;
            assert!(
                (inflow(event, species) - lost).abs() <= 1e-12 * lost.abs().max(1e-30),
                "v{} {species}",
                index + 1
            );
        }
    }
}

fn reservoir_event(drain: bool, inflow_moles: Vec<(SpeciesId, Moles)>) -> Event {
    Event::Transported {
        chain: vec![VesselId(0), VesselId(1), VesselId(2)],
        receiver: VesselId(4),
        steps: 6,
        courant: 0.5,
        effluent_moles: vec![(SpeciesId::new("water"), Moles(3.3206))],
        inflow_moles,
        drain,
        inlet: Some(VesselId(5)),
    }
}

#[test]
fn every_register_and_locale_states_the_reservoir_source_term() {
    let event = reservoir_event(
        false,
        vec![
            (SpeciesId::new("water"), Moles(4.43)),
            (SpeciesId::new("NaCl"), Moles(1e-4)),
        ],
    );
    let cases = [
        ("en", ["constant reservoir", "source term", "4.4300 mol"]),
        (
            "de",
            [
                "Vorratsbehälter konstanter Zusammensetzung",
                "Quellterm",
                "4,4300 mol",
            ],
        ),
        (
            "fr",
            [
                "réservoir à composition constante",
                "terme source",
                "4,4300 mol",
            ],
        ),
    ];
    for (tag, needles) in cases {
        for register in [Register::LV1, Register::LV2, Register::LV3] {
            let line = render_event_in(&event, register, Locale::parse(tag));
            for needle in needles {
                assert!(line.contains(needle), "{tag}: {line}");
            }
            assert!(line.contains("v6"), "{tag}: the inlet is named: {line}");
            assert!(line.contains("100 µmol"), "{tag}: {line}");
        }
    }
}

#[test]
fn a_drained_feed_is_labelled_finite_not_a_reservoir() {
    let event = reservoir_event(true, vec![(SpeciesId::new("water"), Moles(1.1))]);
    let line = render_event_in(&event, Register::LV2, Locale::EN);
    assert!(line.contains("finite feed"), "{line}");
    assert!(!line.contains("constant reservoir"), "{line}");
}

#[test]
fn a_zero_courant_reservoir_reports_that_it_supplied_nothing() {
    let mut bench = tr006(1.0);
    let mut op = transport(3, 2, false);
    if let Operator::Transport { courant, .. } = &mut op {
        *courant = 0.0;
    }
    let events = bench
        .step_with(op, &mut AdsorptionEquilibrator, &PermissiveScreen)
        .unwrap();
    let line = render_event_in(transported(&events), Register::LV3, Locale::EN);
    assert!(line.contains("Inlet v4 is a constant reservoir"), "{line}");
    assert!(line.ends_with("(source term): nothing."), "{line}");
}

#[test]
fn the_event_payload_carries_inlet_and_inflow() {
    let event = reservoir_event(false, vec![(SpeciesId::new("water"), Moles(4.43))]);
    let json = serde_json::to_value(&event).unwrap();
    assert_eq!(json["inlet"], serde_json::json!(5));
    assert!(json["inflow_moles"].is_array());
    assert!(json.get("drain").is_none(), "reservoir is the default mode");
    // A payload from before the inlet was recorded still reads.
    let mut old = json.clone();
    old.as_object_mut().unwrap().remove("inlet");
    let back: Event = serde_json::from_value(old).unwrap();
    assert!(matches!(back, Event::Transported { inlet: None, .. }));
    let line = render_event_in(&back, Register::LV2, Locale::EN);
    assert!(
        line.contains("Inlet (not recorded) is a constant reservoir"),
        "{line}"
    );
}
