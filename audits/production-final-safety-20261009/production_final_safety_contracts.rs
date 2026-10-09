//! Source-informed routing controls using actual reviewed production rules.
//! Synthetic solver changes are injection probes, not chemical predictions.
use kerotakis_core::ops::Endpoint;
use kerotakis_core::solve::{Equilibrator, SafetyScreen, SafetyVerdict, SolveError};
use kerotakis_core::{
    Bench, Event, Liters, Moles, Operator, Phase, Severity, SpeciesId, Vessel, VesselId,
};
use kerotakis_safety::ReactiveGroupScreen;

fn water() -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "production-screen-probe");
    vessel.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    vessel
}
fn add(vessel: &mut Vessel, species: &str) {
    vessel.deposit(SpeciesId::new(species), Moles(0.0001), Phase::Solid);
}
fn warning(verdict: SafetyVerdict, expected_rule: &str, expected_severity: Severity) {
    match verdict {
        SafetyVerdict::Warn {
            rule,
            severity,
            hazard,
            real_world,
        } => {
            assert_eq!(rule, expected_rule);
            assert_eq!(severity, expected_severity);
            assert!(!hazard.is_empty());
            assert!(!real_world.is_empty());
        }
        other => panic!("expected reviewed rule {expected_rule}, got {other:?}"),
    }
}

#[test]
fn newly_introduced_supported_water_reactivity_reaches_final_safety() {
    let before = water();
    let mut after = before.clone();
    add(&mut after, "CaO");
    warning(
        ReactiveGroupScreen.assess_equilibrated(&before, &after),
        "water-reactive-slaking",
        Severity::Caution,
    );
}

#[test]
fn unchanged_preexisting_water_reactivity_is_not_announced_as_new() {
    let mut before = water();
    add(&mut before, "CaO");
    assert!(matches!(
        ReactiveGroupScreen.assess_equilibrated(&before, &before.clone()),
        SafetyVerdict::Allow
    ));
}

#[test]
fn benign_supported_inventory_change_does_not_invent_a_final_rule() {
    let before = water();
    let mut after = before.clone();
    add(&mut after, "NaCl");
    assert!(matches!(
        ReactiveGroupScreen.assess_equilibrated(&before, &after),
        SafetyVerdict::Allow
    ));
}

#[test]
fn a_new_reviewed_finding_is_not_hidden_by_an_older_priority_finding() {
    let mut before = water();
    add(&mut before, "CaO");
    let mut after = before.clone();
    add(&mut after, "NaOCl");
    add(&mut after, "NH3");
    warning(
        ReactiveGroupScreen.assess_equilibrated(&before, &after),
        "bleach-ammonia-chloramine",
        Severity::Danger,
    );
}

#[derive(Clone, Copy)]
enum Injection {
    Always,
    FullDoseOnly,
    None,
}
struct Model {
    injection: Injection,
    calls: usize,
}
impl Equilibrator for Model {
    fn name(&self) -> &'static str {
        "production-final-safety-injection"
    }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.calls += 1;
        let dose = vessel.moles_of(&SpeciesId::new("NaOH")).0;
        if matches!(self.injection, Injection::Always)
            || matches!(self.injection, Injection::FullDoseOnly) && dose > 0.0015
        {
            add(vessel, "CaO");
        }
        vessel.solution = Some(
            serde_json::from_value(serde_json::json!({
                "ph": 4.0 + 1000.0 * dose, "ionic_strength": 0.01
            }))
            .unwrap(),
        );
        Ok(Vec::new())
    }
}
fn bench(preexisting_exposure: bool) -> Bench {
    let mut bench = Bench::new();
    bench.vessels[0] = water();
    if preexisting_exposure {
        add(&mut bench.vessels[0], "CaO");
    }
    bench.vessels[0].solution = Some(
        serde_json::from_value(serde_json::json!({
            "ph": 4.0, "ionic_strength": 0.01
        }))
        .unwrap(),
    );
    bench
}
fn operation(step: f64, target: f64) -> Operator {
    Operator::Titrate {
        vessel: VesselId(0),
        titrant: SpeciesId::new("NaOH"),
        concentration: 1.0,
        step: Liters(step),
        target_ph: target,
        max_steps: 1,
        endpoint: Endpoint::Ph,
    }
}
fn rules(events: &[Event]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::HazardWarning { rule, .. } => Some(rule.as_str()),
            _ => None,
        })
        .collect()
}
fn journal(bench: &Bench, events: &[Event]) {
    assert_eq!(bench.log.len(), 1);
    assert_eq!(bench.log[0].events, events);
}

#[test]
fn actual_production_final_warning_is_returned_and_journaled_once() {
    let mut bench = bench(false);
    let mut model = Model {
        injection: Injection::Always,
        calls: 0,
    };
    let events = bench
        .step_with(operation(0.001, 12.0), &mut model, &ReactiveGroupScreen)
        .unwrap();
    assert_eq!(model.calls, 1);
    assert_eq!(rules(&events), ["water-reactive-slaking"]);
    assert!((bench.vessels[0].moles_of(&SpeciesId::new("CaO")).0 - 0.0001).abs() < 1e-12);
    journal(&bench, &events);
}

#[test]
fn discarded_full_dose_production_warning_does_not_leak_into_refinement() {
    let mut bench = bench(false);
    let mut model = Model {
        injection: Injection::FullDoseOnly,
        calls: 0,
    };
    let events = bench
        .step_with(operation(0.002, 5.0), &mut model, &ReactiveGroupScreen)
        .unwrap();
    assert_eq!(model.calls, 2);
    assert!(rules(&events).is_empty());
    assert_eq!(bench.vessels[0].moles_of(&SpeciesId::new("CaO")).0, 0.0);
    assert!((bench.vessels[0].moles_of(&SpeciesId::new("NaOH")).0 - 0.001).abs() < 1e-12);
    journal(&bench, &events);
}

#[test]
fn preexisting_production_exposure_keeps_only_its_prospective_warning() {
    let mut bench = bench(true);
    let mut model = Model {
        injection: Injection::None,
        calls: 0,
    };
    let events = bench
        .step_with(operation(0.001, 12.0), &mut model, &ReactiveGroupScreen)
        .unwrap();
    assert_eq!(model.calls, 1);
    assert_eq!(rules(&events), ["water-reactive-slaking"]);
    journal(&bench, &events);
}
