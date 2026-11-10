//! Transition-points lane (2026-11-10): the melting- and boiling-point
//! apparatus report a species' sourced constant whatever phase the sample
//! is in, and say which phase that is.
//!
//! Production evidence: a beaker of liquid isopropanol read
//! `nothing_to_test` for its melting point, solid tert-butanol (it melts at
//! 25.8 °C, just above the bench default) read `nothing_to_test` for its
//! boiling point, and liquid dodecane read `nothing_to_test` for its melting
//! point — although the registry carries all three constants. The refusal
//! answered "is there a solid in the tube?", not the question asked.

use kerotakis_core::i18n::Locale;
use kerotakis_core::instrument::{PurityVerdict, TransitionRead};
use kerotakis_core::species::{Phase, TransitionOutcome};
use kerotakis_core::*;

fn vessel_with(contents: &[(&str, f64, Phase)]) -> vessel::Vessel {
    let mut v = vessel::Vessel::new(VesselId(0), "test");
    for (key, moles, phase) in contents {
        v.deposit(SpeciesId::new(key), Moles(*moles), *phase);
    }
    v
}

/// The registry constant in °C, so the tests pin agreement with the data
/// rather than restating numbers.
fn registry_c(key: &str, kind: TransitionRead) -> f64 {
    let t = species::lookup(&SpeciesId::new(key))
        .and_then(|d| d.transitions)
        .unwrap_or_else(|| panic!("{key} has transition data"));
    let (k, _) = match kind {
        TransitionRead::Melting => t.melting_reading(),
        TransitionRead::Boiling => t.boiling_reading(),
    }
    .unwrap_or_else(|| panic!("{key} has a {kind:?} reading"));
    k - 273.15
}

#[test]
fn liquid_isopropanol_reports_its_melting_point_and_says_it_is_liquid() {
    let sample = vessel_with(&[("isopropanol", 0.1, Phase::Liquid)]);
    let r = read_transition(&sample, TransitionRead::Melting);
    assert_eq!(r.verdict, PurityVerdict::Pure);
    assert_eq!(r.sample_phase, Some(Phase::Liquid));
    assert_eq!(r.outcome, Some(TransitionOutcome::Melts));
    let c = r.value_c.unwrap();
    assert!((c - registry_c("isopropanol", TransitionRead::Melting)).abs() < 1e-9);
    assert!(
        c < -80.0 && c > -95.0,
        "isopropanol melts near -88 C, got {c}"
    );
    // Its boiling point still reads through the classic path.
    let b = read_transition(&sample, TransitionRead::Boiling);
    assert_eq!(b.verdict, PurityVerdict::Pure);
    assert_eq!(b.sample_phase, Some(Phase::Liquid));
}

#[test]
fn solid_tert_butanol_reports_its_boiling_point_and_says_it_is_solid() {
    let sample = vessel_with(&[("tert_butanol", 0.1, Phase::Solid)]);
    let r = read_transition(&sample, TransitionRead::Boiling);
    assert_eq!(r.verdict, PurityVerdict::Pure);
    assert_eq!(r.sample_phase, Some(Phase::Solid));
    assert_eq!(r.outcome, Some(TransitionOutcome::Boils));
    let c = r.value_c.unwrap();
    assert!((c - registry_c("tert_butanol", TransitionRead::Boiling)).abs() < 0.05);
    let m = read_transition(&sample, TransitionRead::Melting);
    assert_eq!(m.verdict, PurityVerdict::Pure);
    assert_eq!(m.sample_phase, Some(Phase::Solid));
}

#[test]
fn liquid_dodecane_reports_its_melting_point() {
    let sample = vessel_with(&[("dodecane", 0.05, Phase::Liquid)]);
    let r = read_transition(&sample, TransitionRead::Melting);
    assert_eq!(r.verdict, PurityVerdict::Pure);
    assert_eq!(r.sample_phase, Some(Phase::Liquid));
    let c = r.value_c.unwrap();
    assert!((c - registry_c("dodecane", TransitionRead::Melting)).abs() < 1e-9);
}

#[test]
fn a_species_without_data_gets_the_typed_no_data_reason_not_nothing_to_test() {
    // tert-butyl bromide is a registry liquid with no curated transition.
    let sample = vessel_with(&[("tert_butyl_bromide", 0.05, Phase::Liquid)]);
    let r = read_transition(&sample, TransitionRead::Melting);
    assert_eq!(r.verdict, PurityVerdict::NoData);
    assert!(r.value_c.is_none());
    assert_eq!(r.sample_phase, Some(Phase::Liquid));
    assert_eq!(r.species.as_ref().unwrap().0.as_str(), "tert_butyl_bromide");
}

#[test]
fn a_dissolved_solute_still_makes_the_fallback_sample_not_isolated() {
    let sample = vessel_with(&[
        ("isopropanol", 0.1, Phase::Liquid),
        ("Na+", 0.01, Phase::Aqueous),
    ]);
    let r = read_transition(&sample, TransitionRead::Melting);
    assert_eq!(r.verdict, PurityVerdict::NotIsolated);
    assert!(r.value_c.is_none());
}

#[test]
fn two_liquids_are_still_a_mixture_in_the_fallback() {
    let sample = vessel_with(&[
        ("isopropanol", 0.1, Phase::Liquid),
        ("ethanol", 0.1, Phase::Liquid),
    ]);
    let r = read_transition(&sample, TransitionRead::Melting);
    assert_eq!(r.verdict, PurityVerdict::Mixture);
    assert!(r.value_c.is_none());
}

#[test]
fn the_classic_path_carries_the_wanted_phase_and_no_note() {
    let sample = vessel_with(&[("NaNO3", 0.05, Phase::Solid)]);
    let r = read_transition(&sample, TransitionRead::Melting);
    assert_eq!(r.sample_phase, Some(Phase::Solid));
    let events = vec![Event::TransitionPointRead {
        vessel: VesselId(0),
        reading: r,
    }];
    let lv2 = render_events(&events, Register::LV2).join(" ");
    assert!(!lv2.contains("currently"), "{lv2}");
}

#[test]
fn the_bench_reads_the_melting_point_of_stocked_liquid_isopropanol_in_three_languages() {
    let mut bench = Bench::new();
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("isopropanol"),
            moles: Moles(0.1),
            at: None,
        })
        .expect("add");
    let events = bench
        .step(Operator::Measure {
            vessel: VesselId(0),
            instrument: Instrument::MeltingPointApparatus,
        })
        .expect("step");
    let reading = events
        .iter()
        .find_map(|e| match e {
            Event::TransitionPointRead { reading, .. } => Some(reading),
            _ => None,
        })
        .expect("a transition-point event");
    assert_eq!(reading.verdict, PurityVerdict::Pure);
    assert_eq!(reading.sample_phase, Some(Phase::Liquid));

    let en = render_events(&events, Register::LV2).join(" ");
    assert!(en.contains("currently liquid"), "{en}");
    let de = render_events_in(&events, Register::LV2, Locale::parse("de")).join(" ");
    assert!(de.contains("derzeit flüssig"), "{de}");
    let fr = render_events_in(&events, Register::LV2, Locale::parse("fr")).join(" ");
    assert!(fr.contains("actuellement liquide"), "{fr}");
}
