//! po-009 (2026-11-11, round 4): a thermometer reading in a vessel whose
//! heat accounting skipped a latent heat carries the qualifier ON the
//! reading, not only on the earlier heat step.
//!
//! The predicate is generic: any solid standing above its sourced melting
//! point or the top of its sourced melting range with no enthalpy of fusion
//! the bench can pay (`melting_range::unpaid_melts`). A solid whose melt is
//! paid (paraffin, and since 2026-11-11 every polymer row: grade latent
//! heat = Xc x 100 % reference), a solid below where it melts, and a
//! vessel with no solids read unqualified.

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

/// A sharp-melting solid with a sourced melting point (483 K) and no
/// sourced enthalpy of fusion: the fixture for the unpaid path now that
/// every polymer row pays its grade latent heat (2026-11-11).
const UNPAID: &str = "AgNO3";

#[test]
fn po009_blend_thermometer_reading_is_unqualified_now_the_grade_heat_is_paid() {
    // The original po-009 vessel: 0.5 g PE + 0.5 g PP, 300 J. Both rows
    // now carry a sourced grade latent heat (Xc x 100 % reference), so the
    // heat that melts the polyethylene is paid and the reading is a
    // prediction, not an upper bound.
    let (bench, events) = run(&[
        "add v1 PE 0.5g",
        "add v1 PP 0.5g",
        "heat v1 0.3kJ",
        "measure v1 thermometer",
    ]);
    let (_, note, reason) = reading(&events);
    assert!(note.is_none() && reason.is_none(), "{note:?}");
    let v = bench.vessel(VesselId(0)).unwrap();
    assert!(melting_range::unpaid_melts(v).is_empty());
}

#[test]
fn thermometer_qualifier_names_an_unpaid_solid_and_renders_de_fr() {
    let v = vessel_with(&[(UNPAID, 0.01, Phase::Solid)], 500.0);
    let unpaid = melting_range::unpaid_melts(&v);
    assert_eq!(unpaid.len(), 1, "{unpaid:?}");
    assert!(unpaid[0].sharp_point);
    let reason = melting_range::thermometer_qualifier(&v).expect("qualified");
    assert_eq!(reason.key, KEY);
    let en = reason.render(Locale::EN);
    assert!(en.contains("overstates the warming"), "{en}");
    let de = reason.render(Locale::parse("de"));
    let fr = reason.render(Locale::parse("fr"));
    assert!(de.contains("Schmelzenthalpie"), "{de}");
    assert!(fr.contains("enthalpie de fusion"), "{fr}");
    assert_ne!(de, en);
}

#[test]
fn every_polymer_range_is_paid_so_none_is_named() {
    for t in [430.0, 470.0, 560.0] {
        let v = vessel_with(
            &[
                ("PE", 0.0178, Phase::Solid),
                ("PP", 0.0119, Phase::Solid),
                ("PET", 0.005, Phase::Solid),
                ("nylon", 0.004, Phase::Solid),
            ],
            t,
        );
        let unpaid = melting_range::unpaid_melts(&v);
        assert!(unpaid.is_empty(), "{t} K: {unpaid:?}");
        assert!(melting_range::thermometer_qualifier(&v).is_none());
    }
}

#[test]
fn only_the_unpaid_solid_is_named_beside_a_paid_polymer() {
    let v = vessel_with(
        &[(UNPAID, 0.01, Phase::Solid), ("PE", 0.0178, Phase::Solid)],
        500.0,
    );
    let unpaid = melting_range::unpaid_melts(&v);
    assert_eq!(unpaid.len(), 1, "{unpaid:?}");
    assert_eq!(unpaid[0].species, UNPAID);
}

#[test]
fn step_refusal_and_reading_qualifier_share_one_predicate() {
    for t in [298.15, 430.0, 500.0] {
        let v = vessel_with(
            &[(UNPAID, 0.01, Phase::Solid), ("PE", 0.0178, Phase::Solid)],
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
