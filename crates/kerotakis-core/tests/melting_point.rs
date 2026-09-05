//! EXP-33: the melting-point apparatus, and the refusals that make it
//! worth having.
//!
//! The instrument's value is not that it reports 306.5 °C for sodium
//! nitrate — a table does that. It is that it refuses to report anything at
//! all for a mixture, and says why. These tests pin the refusals at least as
//! hard as the readings.

use kerotakis_core::instrument::{PurityVerdict, TransitionRead};
use kerotakis_core::script::parse_op;
use kerotakis_core::species::TransitionOutcome;
use kerotakis_core::*;

/// A vessel built by hand rather than through `add`, so these tests pin
/// the apparatus and not the dissolution machinery in front of it.
fn vessel_with(contents: &[(&str, f64, species::Phase)]) -> vessel::Vessel {
    let mut v = vessel::Vessel::new(VesselId(0), "test");
    for (key, moles, phase) in contents {
        v.deposit(SpeciesId::new(key), Moles(*moles), *phase);
    }
    v
}

fn read(v: &vessel::Vessel, kind: TransitionRead) -> instrument::TransitionReading {
    read_transition(v, kind)
}

#[test]
fn a_pure_solid_gives_a_sharp_point_with_its_citation() {
    let sample = vessel_with(&[("NaNO3", 0.05, species::Phase::Solid)]);
    let r = read(&sample, TransitionRead::Melting);
    assert_eq!(r.verdict, PurityVerdict::Pure);
    assert_eq!(r.species.as_ref().unwrap().0.as_str(), "NaNO3");
    assert_eq!(r.outcome, Some(TransitionOutcome::Melts));
    let c = r.value_c.expect("a melting point");
    assert!(
        (c - 306.5).abs() < 0.05,
        "sodium nitrate melts at 306.5 C, got {c}"
    );
    // The number never travels without the line that stands behind it.
    let source = r.source.expect("per-record provenance");
    assert!(
        source.contains("phase-transition"),
        "the citation must name the tranche it came from: {source}"
    );
    assert!(
        r.boundary.as_deref().unwrap().contains("does not simulate"),
        "the apparatus must say it is reporting a constant, not a melt"
    );
}

#[test]
fn two_solids_get_a_refusal_and_the_names_of_what_is_in_the_way() {
    // The purity lesson. Sodium nitrate alone melts sharply at 306.5;
    // sodium nitrate with potassium nitrate in it does not melt at 306.5,
    // it does not melt at 334, and it does not melt at any single
    // temperature at all.
    let sample = vessel_with(&[
        ("NaNO3", 0.05, species::Phase::Solid),
        ("KNO3", 0.02, species::Phase::Solid),
    ]);
    let r = read(&sample, TransitionRead::Melting);
    assert_eq!(r.verdict, PurityVerdict::Mixture);
    assert!(
        r.value_c.is_none(),
        "a mixture has no sharp point to report"
    );
    assert_eq!(r.components.len(), 2);
    // The one grounded number: the bound the range begins below.
    let bound = r.lowest_component_c.expect("the lowest pure point");
    assert!((bound - 306.5).abs() < 0.05, "bound was {bound}");
    let boundary = r.boundary.unwrap();
    assert!(
        boundary.contains("SIZE of the depression is not claimed"),
        "the refusal must say which half of the law it is withholding: {boundary}"
    );
}

#[test]
fn a_trace_impurity_is_still_a_mixture_at_this_benchs_threshold() {
    // Deliberately stricter than a real capillary: a tenth of a mole per
    // cent already costs the sharp answer. Erring towards "impure" is the
    // safe direction for an instrument whose whole job is honesty.
    let sample = vessel_with(&[
        ("NaCl", 1.0, species::Phase::Solid),
        ("KCl", 0.002, species::Phase::Solid),
    ]);
    assert_eq!(
        read(&sample, TransitionRead::Melting).verdict,
        PurityVerdict::Mixture
    );

    // Below the threshold it is float dust, not a second substance.
    let sample = vessel_with(&[
        ("NaCl", 1.0, species::Phase::Solid),
        ("KCl", 1e-6, species::Phase::Solid),
    ]);
    assert_eq!(
        read(&sample, TransitionRead::Melting).verdict,
        PurityVerdict::Pure
    );
}

#[test]
fn a_wet_sample_is_refused_rather_than_read_low() {
    // A damp capillary is the classic way to get a low, broad, wrong
    // answer. The bench will not pretend the water is not there.
    let sample = vessel_with(&[
        ("NaCl", 0.05, species::Phase::Solid),
        ("water", 0.01, species::Phase::Liquid),
    ]);
    let r = read(&sample, TransitionRead::Melting);
    assert_eq!(r.verdict, PurityVerdict::NotIsolated);
    assert!(r.value_c.is_none());
    assert_eq!(r.species.as_ref().unwrap().0.as_str(), "NaCl");
}

#[test]
fn a_substance_that_decomposes_is_told_that_it_decomposes() {
    // Sucrose has no melting point. Reporting 186 °C as one would teach
    // the single commonest lie in the school data tables.
    let sample = vessel_with(&[("sucrose", 0.01, species::Phase::Solid)]);
    let r = read(&sample, TransitionRead::Melting);
    assert_eq!(r.verdict, PurityVerdict::Pure);
    assert_eq!(r.outcome, Some(TransitionOutcome::Decomposes));
    assert!((r.value_c.unwrap() - 186.0).abs() < 0.05);
    assert!(r.boundary.unwrap().contains("does not melt"));
}

#[test]
fn a_substance_that_sublimes_is_told_that_it_sublimes() {
    let sample = vessel_with(&[("NH4Cl", 0.01, species::Phase::Solid)]);
    let r = read(&sample, TransitionRead::Melting);
    assert_eq!(r.outcome, Some(TransitionOutcome::Sublimes));
    assert!((r.value_c.unwrap() - 338.0).abs() < 0.05);
}

#[test]
fn a_hydrate_reports_losing_its_water_not_melting() {
    let sample = vessel_with(&[("chalcanthite", 0.01, species::Phase::Solid)]);
    let r = read(&sample, TransitionRead::Melting);
    assert_eq!(r.outcome, Some(TransitionOutcome::LosesWater));
    assert!(r.boundary.unwrap().contains("intermediate hydrates"));
}

#[test]
fn an_uncurated_solid_says_so_instead_of_inventing_a_number() {
    let sample = vessel_with(&[("Na2S2O3", 0.01, species::Phase::Solid)]);
    let r = read(&sample, TransitionRead::Melting);
    assert_eq!(r.verdict, PurityVerdict::NoData);
    assert!(r.value_c.is_none());
}

#[test]
fn an_empty_vessel_has_nothing_to_test() {
    let sample = vessel_with(&[]);
    assert_eq!(
        read(&sample, TransitionRead::Melting).verdict,
        PurityVerdict::NothingToTest
    );
}

#[test]
fn the_boiling_apparatus_reads_liquids_and_agrees_with_the_states_model() {
    let sample = vessel_with(&[("water", 1.0, species::Phase::Liquid)]);
    let r = read(&sample, TransitionRead::Boiling);
    assert_eq!(r.verdict, PurityVerdict::Pure);
    let c = r.value_c.unwrap();
    // The registry row and the colligative model must not disagree about
    // where pure water boils; two numbers for one fact is a bug waiting.
    assert!(
        (c + 273.15 - kerotakis_core::states::WATER_BOILING_K).abs() < 1e-9,
        "registry says {c} C, states.rs says {} K",
        kerotakis_core::states::WATER_BOILING_K
    );

    let sample = vessel_with(&[("ethanol", 1.0, species::Phase::Liquid)]);
    let c = read(&sample, TransitionRead::Boiling).value_c.unwrap();
    assert!(
        (c - 78.29).abs() < 0.05,
        "ethanol boils at 78.29 C, got {c}"
    );
}

#[test]
fn a_solution_is_not_a_pure_liquid() {
    // Salt water boils above 100 °C, and by how much is a colligative
    // question the states model answers — not something to read off a
    // table of pure-substance constants.
    let sample = vessel_with(&[
        ("water", 1.0, species::Phase::Liquid),
        ("NaCl", 0.05, species::Phase::Solid),
    ]);
    assert_eq!(
        read(&sample, TransitionRead::Boiling).verdict,
        PurityVerdict::NotIsolated
    );
}

#[test]
fn the_grammar_takes_the_quantity_and_the_apparatus_and_rejects_nonsense() {
    for line in [
        "measure v1 melting_point",
        "measure v1 mp",
        "measure v1 melting-point",
    ] {
        assert!(
            matches!(
                parse_op(line),
                Ok(Some(Operator::Measure {
                    instrument: Instrument::MeltingPointApparatus,
                    ..
                }))
            ),
            "{line} should parse to the melting-point apparatus"
        );
    }
    for line in ["measure v1 boiling_point", "measure v1 bp"] {
        assert!(matches!(
            parse_op(line),
            Ok(Some(Operator::Measure {
                instrument: Instrument::BoilingPointApparatus,
                ..
            }))
        ));
    }
    // Near misses must be refused, not silently routed somewhere.
    for line in [
        "measure v1 melting",
        "measure v1 meltingpoint",
        "measure v1 m p",
    ] {
        assert!(parse_op(line).is_err(), "{line} should not parse");
    }
    // The parser never panics on rubbish: the fuzz target's invariant,
    // pinned here for the tokens this task added.
    for line in [
        "measure",
        "measure v1",
        "measure v1 mp extra",
        "measure v1 MP",
    ] {
        let _ = parse_op(line);
    }
}

#[test]
fn the_bench_emits_the_reading_as_an_event() {
    let mut bench = Bench::new();
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("NaNO3"),
            moles: Moles(0.05),
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

    // Every register renders it, and lv3 carries the citation.
    for register in [Register::LV1, Register::LV2, Register::LV3] {
        let text = render_events(&events, register).join("\n");
        assert!(
            !text.is_empty(),
            "register {} rendered nothing",
            register.level()
        );
    }
    let lv3 = render_events(&events, Register::LV3).join("\n");
    assert!(
        lv3.contains("phase-transition"),
        "lv3 must print the source: {lv3}"
    );
}

// BRD-032: the boiling-point apparatus reads at the vessel's own pressure.

fn water_inchikey() -> &'static str {
    species::lookup(&SpeciesId::new("water")).unwrap().inchikey
}

#[test]
fn under_vacuum_the_boiling_apparatus_reads_lower_and_names_the_model() {
    let mut sample = vessel_with(&[("water", 1.0, species::Phase::Liquid)]);
    sample.pressure = Pascal(50_000.0);
    let r = read(&sample, TransitionRead::Boiling);
    assert_eq!(r.verdict, PurityVerdict::Pure);
    let c = r.value_c.unwrap();
    // Steam tables: 81.3 °C at 50 kPa.
    assert!(
        (c - 81.35).abs() < 0.3,
        "water at 50 kPa boils near 81.3 C, got {c}"
    );
    let source = r.source.unwrap();
    assert!(
        source.contains("shifted") && source.contains("50.00 kPa"),
        "{source}"
    );
    assert!(
        kerotakis_thermo::pack::row_by_inchikey(water_inchikey()).is_some(),
        "the shift came from the pack row the species' InChIKey names"
    );
}

#[test]
fn at_one_atmosphere_the_apparatus_reads_exactly_what_it_always_did() {
    let sample = vessel_with(&[("water", 1.0, species::Phase::Liquid)]);
    assert_eq!(sample.pressure, Pascal::ATMOSPHERIC);
    let r = read(&sample, TransitionRead::Boiling);
    let c = r.value_c.unwrap();
    assert!((c + 273.15 - kerotakis_core::states::WATER_BOILING_K).abs() < 1e-9);
    let source = r.source.unwrap();
    assert!(
        !source.contains("shifted"),
        "no correlation is consulted at 1 atm: {source}"
    );
    assert!(!r
        .boundary
        .unwrap()
        .contains("normal boiling point although"));
}

#[test]
fn outside_the_cleared_window_the_curated_value_stands_and_says_so() {
    // A pressure cooker: above water's cleared fit. The curated point is
    // read, and the reading says the pressure was not answered.
    let mut sample = vessel_with(&[("water", 1.0, species::Phase::Liquid)]);
    sample.pressure = Pascal(300_000.0);
    let r = read(&sample, TransitionRead::Boiling);
    let c = r.value_c.unwrap();
    assert!((c + 273.15 - kerotakis_core::states::WATER_BOILING_K).abs() < 1e-9);
    let boundary = r.boundary.unwrap();
    assert!(
        boundary.contains("300.00 kPa") && boundary.contains("pressure-outside-cleared-window"),
        "{boundary}"
    );
}

#[test]
fn a_liquid_the_pack_does_not_know_is_read_at_its_normal_point_and_told_why() {
    // Every cleared fluid answers or refuses by name; a liquid with no pack
    // row is told the pack does not know it, and keeps its curated point.
    let mut sample = vessel_with(&[("ethanol", 1.0, species::Phase::Liquid)]);
    sample.pressure = Pascal(50_000.0);
    let r = read(&sample, TransitionRead::Boiling);
    let c = r.value_c.unwrap();
    let key = species::lookup(&SpeciesId::new("ethanol"))
        .unwrap()
        .inchikey;
    match kerotakis_thermo::pack::row_by_inchikey(key) {
        Some(row) if row.boiling_point_c_at(50.0).is_ok() => {
            assert!(
                c < 78.0,
                "ethanol under vacuum boils below 78.29 C, got {c}"
            );
            assert!(r.source.unwrap().contains("shifted"));
        }
        _ => {
            assert!((c - 78.29).abs() < 0.05, "the curated point stands: {c}");
            let boundary = r.boundary.unwrap();
            assert!(
                boundary.contains("solvent-not-in-pack")
                    || boundary.contains("pressure-outside-cleared-window"),
                "{boundary}"
            );
        }
    }
}
