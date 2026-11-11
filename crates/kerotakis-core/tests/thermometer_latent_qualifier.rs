//! po-009 (2026-11-11, round 4): a thermometer reading in a vessel whose
//! heat accounting skipped a latent heat carries the qualifier ON the
//! reading, not only on the earlier heat step.
//!
//! The predicate is generic: any solid standing above its sourced melting
//! point or the top of its sourced melting range with no enthalpy of fusion
//! the bench can pay (`melting_range::unpaid_melts`). A solid whose melt is
//! paid (paraffin), a solid below where it melts, and a vessel with no
//! solids read unqualified.

use kerotakis_core::melting_range;
use kerotakis_core::script::parse_op;
use kerotakis_core::species::Phase;
use kerotakis_core::*;

const KEY: &str = "measured.thermometer-latent-heat-unpaid";

fn run(commands: &[&str]) -> (Bench, Vec<Event>) {
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(StateEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(PhaseRouteEquilibrator),
        Box::new(HonestyEquilibrator),
    ]);
    let mut last = Vec::new();
    for command in commands {
        let op = parse_op(command)
            .unwrap_or_else(|e| panic!("parse {command}: {e}"))
            .expect("operator");
        last = bench
            .step_with(op, &mut stack, &PermissiveScreen)
            .unwrap_or_else(|e| panic!("{command}: {e}"));
    }
    (bench, last)
}

/// The thermometer reading of the last step, with its qualifier.
fn reading(events: &[Event]) -> (f64, Option<String>, Option<phrase::Phrase>) {
    events
        .iter()
        .find_map(|e| match e {
            Event::Measured {
                instrument: Instrument::Thermometer,
                value,
                note,
                note_reason,
                ..
            } => Some((*value, note.clone(), note_reason.clone())),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no thermometer reading: {events:?}"))
}

fn vessel_with(contents: &[(&str, f64, Phase)], t_k: f64) -> vessel::Vessel {
    let mut v = vessel::Vessel::new(VesselId(0), "test");
    for (key, moles, phase) in contents {
        v.deposit(SpeciesId::new(key), Moles(*moles), *phase);
    }
    v.temperature = Kelvin(t_k);
    v
}

#[test]
fn po009_blend_thermometer_reading_carries_the_unpaid_latent_heat_qualifier() {
    // The original po-009 vessel: 0.5 g PE + 0.5 g PP, 300 J. Sensible heat
    // alone carries it past both sourced upper ends (141.45, 187.55 °C).
    let (_, events) = run(&[
        "add v1 PE 0.5g",
        "add v1 PP 0.5g",
        "heat v1 0.3kJ",
        "measure v1 thermometer",
    ]);
    let (value, note, reason) = reading(&events);
    assert!(value > 187.55, "the uncorrected reading: {value} °C");
    let reason = reason.expect("qualified reading");
    assert_eq!(reason.key, KEY);
    let en = note.expect("English note");
    assert_eq!(en, reason.render(Locale::EN));
    assert!(en.contains("overstates the warming"), "{en}");
    assert!(en.contains("not a prediction"), "{en}");
}

#[test]
fn thermometer_qualifier_names_every_unpaid_polymer_and_renders_de_fr() {
    let pp = melting_range::range("PP").expect("PP range");
    let v = vessel_with(
        &[("PE", 0.0178, Phase::Solid), ("PP", 0.0119, Phase::Solid)],
        pp.upper_k + 5.0,
    );
    let unpaid = melting_range::unpaid_melts(&v);
    assert_eq!(unpaid.len(), 2, "{unpaid:?}");
    assert!(unpaid.iter().all(|u| !u.sharp_point));
    let reason = melting_range::thermometer_qualifier(&v).expect("qualified");
    assert_eq!(reason.key, KEY);
    let de = reason.render(Locale::parse("de"));
    let fr = reason.render(Locale::parse("fr"));
    assert!(de.contains("Schmelzenthalpie"), "{de}");
    assert!(fr.contains("enthalpie de fusion"), "{fr}");
    assert_ne!(de, reason.render(Locale::EN));
}

#[test]
fn only_the_polymer_above_its_range_is_named() {
    // Between PE's (414.6 K) and PP's (460.7 K) upper ends: only PE.
    let v = vessel_with(
        &[("PE", 0.0178, Phase::Solid), ("PP", 0.0119, Phase::Solid)],
        430.0,
    );
    let unpaid = melting_range::unpaid_melts(&v);
    assert_eq!(unpaid.len(), 1, "{unpaid:?}");
    assert_eq!(unpaid[0].species, "PE");
}

#[test]
fn step_refusal_and_reading_qualifier_share_one_predicate() {
    for t in [298.15, 430.0, 470.0] {
        let v = vessel_with(
            &[("PE", 0.0178, Phase::Solid), ("PP", 0.0119, Phase::Solid)],
            t,
        );
        let refusals = melting_range::unmodelled_melts(&v).len();
        let unpaid = melting_range::unpaid_melts(&v).len();
        assert_eq!(refusals, unpaid, "{t} K");
        assert_eq!(
            melting_range::thermometer_qualifier(&v).is_some(),
            unpaid > 0,
            "{t} K"
        );
    }
}

#[test]
fn a_polymer_below_its_range_reads_unqualified() {
    let (_, events) = run(&["add v1 PE 1g", "measure v1 thermometer"]);
    let (_, note, reason) = reading(&events);
    assert!(note.is_none() && reason.is_none(), "{note:?}");
}

#[test]
fn a_paid_wax_melt_reads_unqualified() {
    // Paraffin's sourced 189 kJ/kg is paid through its range: 2 kJ into 5 g
    // melts it all and the reading needs no qualifier.
    let (bench, events) = run(&[
        "add v1 paraffin 5g @ 20C",
        "heat v1 2kJ",
        "measure v1 thermometer",
    ]);
    let (_, note, reason) = reading(&events);
    assert!(note.is_none() && reason.is_none(), "{note:?}");
    let v = bench.vessel(VesselId(0)).unwrap();
    assert!(melting_range::unpaid_melts(v).is_empty());
}

#[test]
fn heated_water_reads_unqualified() {
    let (_, events) = run(&["add v1 water 50mL", "heat v1 2kJ", "measure v1 thermometer"]);
    let (_, note, reason) = reading(&events);
    assert!(note.is_none() && reason.is_none(), "{note:?}");
}

#[test]
fn a_melted_polymer_liquid_is_not_named() {
    let pe = melting_range::range("PE").expect("PE range");
    let v = vessel_with(&[("PE", 0.0178, Phase::Liquid)], pe.upper_k + 30.0);
    assert!(melting_range::thermometer_qualifier(&v).is_none());
}
