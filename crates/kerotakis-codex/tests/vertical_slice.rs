//! WORLD-008 — the vertical slice, end to end.
//!
//! "The contaminated sample" encoded as v2 quests and run through the real
//! engine: three leads open at once, a sealed unknown, two materially
//! different treatment traces, and one permanent unlock derived from the
//! case rather than granted by a ledger.
//!
//! The fixtures carry i18n KEYS rather than English sentences. That is
//! deliberate: content whose prose lives per-locale cannot ship an
//! untranslated string, which is the direction WORLD-007's ratchet points.

use std::collections::BTreeMap;

use kerotakis_codex::quest::{self, QuestOutput, QuestSpec, QuestState};
use kerotakis_core::*;

const CASE: &str = "the-contaminated-sample";

fn case_specs() -> Vec<QuestSpec> {
    let dir = std::path::Path::new("tests/fixtures/contaminated-sample");
    let specs = quest::load_dir(dir).expect("the case fixtures load");
    assert_eq!(specs.len(), 3, "three concurrent leads");
    specs
}

fn fresh_states(specs: &[QuestSpec]) -> BTreeMap<String, QuestState> {
    specs
        .iter()
        .map(|s| (s.id.clone(), QuestState::default()))
        .collect()
}

fn peak(species: &str, retention_time_s: f64, width_s: f64) -> ops::ElutedPeak {
    ops::ElutedPeak {
        species: SpeciesId::new(species),
        retention_time_s,
        width_s,
        relative_area: 1.0,
        partition_k: 1.0,
        // KID-9: the same K read as a paper strip reads it.
        rf: 0.5,
    }
}

/// The column trace: one injection the school column resolves into three.
fn column_trace() -> Vec<Event> {
    vec![Event::Chromatographed {
        vessel: VesselId(0),
        plates: 10_000,
        void_time_s: 30.0,
        peaks: vec![
            peak("methanol", 63.0, 2.5),
            peak("ethanol", 68.0, 2.7),
            peak("propanone", 115.0, 4.6),
        ],
        outside_method: vec![],
    }]
}

/// The funnel trace: the engine's own partition numbers for a real,
/// sample-sized extraction (spread 0.190), and the layer drawn off.
fn funnel_trace() -> Vec<Event> {
    vec![
        Event::Partitioned {
            vessel: VesselId(0),
            species: SpeciesId::new("methanol"),
            fraction_lower: 0.9884828398233436,
        },
        Event::Partitioned {
            vessel: VesselId(0),
            species: SpeciesId::new("ethanol"),
            fraction_lower: 0.9632931020927237,
        },
        Event::Partitioned {
            vessel: VesselId(0),
            species: SpeciesId::new("propanone"),
            fraction_lower: 0.7980447115886924,
        },
        Event::Drained {
            from: VesselId(0),
            to: VesselId(1),
            solvent: SpeciesId::new("water"),
            moles: Moles(5.4),
        },
    ]
}

#[test]
fn the_case_fixtures_pass_the_lint_they_will_be_authored_against() {
    let specs = case_specs();
    let problems = quest::lint(&specs);
    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn all_three_leads_are_placed_in_one_chapter_and_open_together() {
    let specs = case_specs();
    for spec in &specs {
        let placement = spec
            .placement
            .as_ref()
            .unwrap_or_else(|| panic!("{} has no placement", spec.id));
        assert_eq!(placement.district, "discovery-hall");
        assert_eq!(placement.chapter.as_deref(), Some(CASE));
    }
    // Three distinct orders, so the board can lay them out deterministically
    // without inventing a sort.
    let mut orders: Vec<u32> = specs
        .iter()
        .map(|s| s.placement.as_ref().unwrap().order.unwrap())
        .collect();
    orders.sort_unstable();
    assert_eq!(orders, vec![1, 2, 3]);
}

#[test]
fn the_three_leads_progress_concurrently_from_one_bench() {
    let specs = case_specs();
    let mut states = fresh_states(&specs);
    let bench = Bench::new();

    // One step of evidence that belongs to lead three only.
    quest::observe(&specs, &mut states, &column_trace(), &bench);
    assert!(states["separate-the-mixture"]
        .satisfied
        .contains("ran-the-column"));
    // The other two leads are untouched — not blocked, not advanced, and not
    // completed by a sibling's evidence. That is what concurrent means here.
    assert!(states["trace-the-contamination"].satisfied.is_empty());
    assert!(states["establish-the-baseline"].satisfied.is_empty());
    assert!(!states["trace-the-contamination"].complete);
    assert!(!states["establish-the-baseline"].complete);
    // Lead three, meanwhile, closed on that single step: the column trace
    // carries both its outcome and its own physical evidence at once.
    assert!(states["separate-the-mixture"].complete);

    // Evidence for lead one, in the same session, in any order.
    let precipitate = vec![Event::Precipitated {
        vessel: VesselId(0),
        species: SpeciesId::new("AgCl"),
        moles: Moles(0.01),
    }];
    quest::observe(&specs, &mut states, &precipitate, &bench);
    assert!(states["trace-the-contamination"]
        .satisfied
        .contains("visible-precipitate"));
    // Still not complete: the sealed unknown has not been named.
    assert!(!states["trace-the-contamination"].complete);
}

#[test]
fn the_sealed_unknown_closes_the_lead_only_when_it_is_named() {
    let specs = case_specs();
    let mut states = fresh_states(&specs);
    let bench = Bench::new();
    quest::observe(
        &specs,
        &mut states,
        &[Event::Precipitated {
            vessel: VesselId(0),
            species: SpeciesId::new("AgCl"),
            moles: Moles(0.01),
        }],
        &bench,
    );

    // A wrong name is answered, not punished: a spoken refusal that closes
    // nothing and locks nothing.
    let refusal = quest::answer(&specs, &mut states, "field-sample", "KCl")
        .expect_err("a wrong guess is spoken, not accepted");
    assert!(refusal.contains("not"), "spoken refusal: {refusal}");
    assert!(!states["trace-the-contamination"].complete);

    let out = quest::answer(&specs, &mut states, "field-sample", "NaCl").expect("answered");
    assert!(out
        .iter()
        .any(|o| matches!(o, QuestOutput::Completed { .. })));
    assert!(states["trace-the-contamination"].complete);
}

#[test]
fn two_materially_different_treatment_traces_both_close_the_same_lead() {
    let specs = case_specs();
    let bench = Bench::new();

    // Trace one: the column, and no funnel anywhere.
    let mut column = fresh_states(&specs);
    quest::observe(&specs, &mut column, &column_trace(), &bench);
    let spec = specs
        .iter()
        .find(|s| s.id == "separate-the-mixture")
        .unwrap();
    assert!(column["separate-the-mixture"].complete);
    assert_eq!(
        spec.completed_route(&column["separate-the-mixture"])
            .map(|r| r.id.as_str()),
        Some("on-the-column")
    );

    // Trace two: the funnel, and no chromatogram anywhere.
    let mut funnel = fresh_states(&specs);
    quest::observe(&specs, &mut funnel, &funnel_trace(), &bench);
    assert!(funnel["separate-the-mixture"].complete);
    assert_eq!(
        spec.completed_route(&funnel["separate-the-mixture"])
            .map(|r| r.id.as_str()),
        Some("in-the-funnel")
    );

    // Neither run needed the optional discovery.
    assert!(!column["separate-the-mixture"]
        .satisfied
        .contains("looked-first"));
    assert!(!funnel["separate-the-mixture"]
        .satisfied
        .contains("looked-first"));
}

#[test]
fn each_trace_needs_its_own_physical_evidence() {
    let specs = case_specs();
    let spec = specs
        .iter()
        .find(|s| s.id == "separate-the-mixture")
        .unwrap();
    let bench = Bench::new();

    // The two routes share an outcome and nothing else: neither route's
    // own evidence is produced by the other's work, which is what stops a
    // learner closing the funnel route with a column run.
    let mut column = fresh_states(&specs);
    quest::observe(&specs, &mut column, &column_trace(), &bench);
    let column_state = &column["separate-the-mixture"];
    assert!(column_state.satisfied.contains("ran-the-column"));
    assert!(!column_state.satisfied.contains("drew-off-the-layer"));

    let mut funnel = fresh_states(&specs);
    quest::observe(&specs, &mut funnel, &funnel_trace(), &bench);
    let funnel_state = &funnel["separate-the-mixture"];
    assert!(funnel_state.satisfied.contains("drew-off-the-layer"));
    assert!(!funnel_state.satisfied.contains("ran-the-column"));

    // And a funnel that never drew the layer off separated nothing at all:
    // the outcome itself is physical, not arithmetic over partition numbers.
    let mut unfinished = fresh_states(&specs);
    let mut events = funnel_trace();
    events.retain(|e| !matches!(e, Event::Drained { .. }));
    quest::observe(&specs, &mut unfinished, &events, &bench);
    let unfinished_state = &unfinished["separate-the-mixture"];
    assert!(!unfinished_state.satisfied.contains("told-apart"));
    assert!(!unfinished_state.complete);
    assert!(spec.completed_route(unfinished_state).is_none());
}

#[test]
fn the_constraint_is_recorded_without_closing_the_door() {
    let specs = case_specs();
    let mut states = fresh_states(&specs);
    let bench = Bench::new();

    quest::observe(
        &specs,
        &mut states,
        &[Event::Evaporated {
            vessel: VesselId(0),
            moles: Moles(1.0),
        }],
        &bench,
    );
    assert!(states["separate-the-mixture"]
        .violated
        .contains("did-not-boil-it-dry"));

    // And the lead still finishes afterwards: the mistake is the lesson.
    quest::observe(&specs, &mut states, &column_trace(), &bench);
    assert!(states["separate-the-mixture"].complete);
}

#[test]
fn the_case_grants_exactly_one_permanent_unlock_however_it_is_reached() {
    let specs = case_specs();
    // The reward is DECLARED once, on the lead that closes the case, and it
    // names a catalog id in WORLD-003's id space. Nothing accumulates it:
    // asking twice returns the same single answer.
    let rewards: Vec<(&str, &str)> = specs
        .iter()
        .flat_map(|s| s.rewards.iter().map(|r| (r.kind.as_str(), r.id.as_str())))
        .collect();
    assert_eq!(rewards, vec![("equipment", "measure:uvvis")]);
    assert_eq!(
        kerotakis_core::catalog::equipment_requirement("measure:uvvis"),
        3,
        "the award is worth having: three missions would otherwise be needed"
    );
}

#[test]
fn the_fixtures_carry_translation_keys_rather_than_untranslated_prose() {
    // WORLD-007's ratchet counts English quest prose. Content whose strings
    // are keys cannot add to that debt, which is what makes this slice
    // shippable in two languages on the day it lands.
    for spec in case_specs() {
        for text in [&spec.title.lv1, &spec.goal.lv1] {
            assert!(
                !text.contains(' '),
                "{}: fixture prose should be a key, got {text:?}",
                spec.id
            );
        }
        for claim in &spec.claims {
            assert!(
                !claim.title.lv1.contains(' '),
                "{}: claim {} title should be a key",
                spec.id,
                claim.id
            );
        }
    }
}
