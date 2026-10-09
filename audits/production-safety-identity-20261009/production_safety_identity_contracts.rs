//! Source-informed identity/routing controls, not chemical transformation forecasts.
use kerotakis_core::solve::{SafetyScreen, SafetyVerdict};
use kerotakis_core::{Moles, Phase, Severity, SpeciesId, Vessel, VesselId};
use kerotakis_safety::ReactiveGroupScreen;

fn probe(keys: &[&str]) -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "safety-identity-probe");
    vessel.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    for key in keys {
        vessel.deposit(SpeciesId::new(*key), Moles(0.0001), Phase::Liquid);
    }
    vessel
}
fn allow(before: &Vessel, after: &Vessel) {
    assert!(matches!(
        ReactiveGroupScreen.assess_equilibrated(before, after),
        SafetyVerdict::Allow
    ));
}
fn warn(before: &Vessel, after: &Vessel, rule_id: &str, level: Severity) {
    match ReactiveGroupScreen.assess_equilibrated(before, after) {
        SafetyVerdict::Warn {
            severity,
            rule,
            hazard,
            real_world,
        } => {
            assert_eq!(rule, rule_id);
            assert_eq!(severity, level);
            assert!(!hazard.is_empty() && !real_world.is_empty());
        }
        other => panic!("expected reviewed {rule_id} warning, got {other:?}"),
    }
}

#[test]
fn hypochlorite_ion_projection_does_not_repeat_existing_chloramine_warning() {
    allow(&probe(&["NaOCl", "NH3"]), &probe(&["ClO-", "NH3"]));
}
#[test]
fn hypochlorous_projection_does_not_repeat_existing_chloramine_warning() {
    allow(&probe(&["NaOCl", "NH3"]), &probe(&["HClO", "NH3"]));
}
#[test]
fn permanganate_ion_projection_does_not_repeat_existing_fire_warning() {
    allow(&probe(&["KMnO4", "ethanol"]), &probe(&["MnO4-", "ethanol"]));
}
#[test]
fn a_distinct_oxidizer_pair_is_not_hidden_by_an_existing_rule_id() {
    let before = probe(&["KMnO4", "ethanol"]);
    let mut after = before.clone();
    after.deposit(SpeciesId::new("H2O2"), Moles(0.0001), Phase::Liquid);
    warn(
        &before,
        &after,
        "oxidizer-flammable-liquid",
        Severity::Danger,
    );
}
#[test]
fn a_distinct_fuel_pair_is_not_hidden_by_an_existing_rule_id() {
    let before = probe(&["H2O2", "ethanol"]);
    let mut after = before.clone();
    after.deposit(SpeciesId::new("isopropanol"), Moles(0.0001), Phase::Liquid);
    warn(
        &before,
        &after,
        "oxidizer-flammable-liquid",
        Severity::Danger,
    );
}
#[test]
fn reordering_existing_species_does_not_create_a_new_finding() {
    let before = probe(&["CaO"]);
    let mut after = before.clone();
    after.contents.reverse();
    allow(&before, &after);
}
#[test]
fn at_or_below_existing_screen_cutoff_does_not_create_a_final_finding() {
    let before = probe(&[]);
    for amount in [5e-13, 1e-12] {
        let mut after = before.clone();
        after.deposit(SpeciesId::new("CaO"), Moles(amount), Phase::Solid);
        allow(&before, &after);
    }
}
#[test]
fn above_existing_screen_cutoff_reaches_the_final_warning() {
    let before = probe(&[]);
    let mut after = before.clone();
    after.deposit(SpeciesId::new("CaO"), Moles(2e-12), Phase::Solid);
    warn(&before, &after, "water-reactive-slaking", Severity::Caution);
}
#[test]
fn introduced_warning_preserves_the_exact_reviewed_metadata() {
    let before = probe(&[]);
    let after = probe(&["CaO"]);
    let fields = |verdict| match verdict {
        SafetyVerdict::Warn {
            severity,
            rule,
            hazard,
            real_world,
        } => (severity, rule, hazard, real_world),
        other => panic!("expected reviewed warning, got {other:?}"),
    };
    assert_eq!(
        fields(ReactiveGroupScreen.assess_equilibrated(&before, &after)),
        fields(ReactiveGroupScreen.assess(&after))
    );
}
