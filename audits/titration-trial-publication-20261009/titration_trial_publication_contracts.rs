//! Frozen source-informed publication controls using a synthetic endpoint model.
//! The linear coordinate below is not an aqueous chemistry or pH forecast.
use kerotakis_core::ops::Endpoint;
use kerotakis_core::solve::{Equilibrator, SafetyScreen, SafetyVerdict, SolveError};
use kerotakis_core::{Bench, Event, Liters, Moles, Operator, Phase, SpeciesId, Vessel, VesselId};
use std::cell::Cell;

#[derive(Default)]
struct Model {
    calls: usize,
    invalid_at: Option<usize>,
    fail_at: Option<usize>,
}
impl Equilibrator for Model {
    fn name(&self) -> &'static str {
        "synthetic-publication-coordinate"
    }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.calls += 1;
        if self.invalid_at == Some(self.calls) {
            vessel.contents[0].moles = Moles(-1.0);
        }
        if self.fail_at == Some(self.calls) {
            // Deliberately corrupt the private trial before returning failure.
            vessel.contents[0].moles = Moles(123.0);
            return Err(SolveError::NotConverged {
                solver: self.name().into(),
                detail: "frozen failure after candidate mutation".into(),
            });
        }
        let coordinate = 4.0 + 1000.0 * vessel.moles_of(&SpeciesId::new("NaOH")).0;
        vessel.solution = Some(
            serde_json::from_value(serde_json::json!({
                "ph": coordinate, "ionic_strength": 0.01
            }))
            .unwrap(),
        );
        Ok(Vec::new())
    }
}
#[derive(Default)]
struct Screen {
    prospective_calls: Cell<usize>,
    final_calls: Cell<usize>,
    prospective_veto_above: Option<f64>,
    final_veto: bool,
    refinement_veto: bool,
    final_warn: bool,
}
impl SafetyScreen for Screen {
    fn assess(&self, vessel: &Vessel) -> SafetyVerdict {
        self.prospective_calls.set(self.prospective_calls.get() + 1);
        if self
            .prospective_veto_above
            .is_some_and(|limit| vessel.moles_of(&SpeciesId::new("NaOH")).0 > limit)
        {
            SafetyVerdict::Veto {
                reason: "frozen later prospective veto".into(),
            }
        } else {
            SafetyVerdict::Allow
        }
    }
    fn assess_equilibrated(&self, before: &Vessel, after: &Vessel) -> SafetyVerdict {
        self.final_calls.set(self.final_calls.get() + 1);
        assert!(
            before.solution.is_none(),
            "before must be the unsolved trial"
        );
        assert!(after.solution.is_some(), "after must be the solved trial");
        let amount = after.moles_of(&SpeciesId::new("NaOH")).0;
        if self.final_veto || self.refinement_veto && (amount - 0.001).abs() < 1e-12 {
            SafetyVerdict::Veto {
                reason: "frozen settled-trial veto".into(),
            }
        } else if self.final_warn {
            SafetyVerdict::Warn {
                severity: kerotakis_core::Severity::Danger,
                rule: format!("settled-dose-{amount:.6}"),
                hazard: "synthetic final warning".into(),
                real_world: "failure-injection fixture".into(),
            }
        } else {
            SafetyVerdict::Allow
        }
    }
}
fn bench() -> Bench {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    bench.vessels[0].solution = Some(
        serde_json::from_value(serde_json::json!({"ph": 4.0, "ionic_strength": 0.01})).unwrap(),
    );
    bench
}
fn operation(step: f64, target: f64, max_steps: u32) -> Operator {
    Operator::Titrate {
        vessel: VesselId(0),
        titrant: SpeciesId::new("NaOH"),
        concentration: 1.0,
        step: Liters(step),
        target_ph: target,
        max_steps,
        endpoint: Endpoint::Ph,
    }
}
fn physical(bench: &Bench) -> String {
    let mut checkpoint = bench.clone();
    checkpoint.log.clear();
    format!("{checkpoint:?}")
}
fn veto_journal(bench: &Bench, events: &[Event]) {
    assert!(matches!(events, [Event::SafetyVeto { .. }]));
    assert_eq!(bench.log.len(), 1);
    assert_eq!(bench.log[0].events, events);
}

#[test]
fn settled_first_trial_veto_rolls_back_the_whole_operation() {
    let mut bench = bench();
    let before = physical(&bench);
    let mut model = Model::default();
    let screen = Screen {
        final_veto: true,
        ..Screen::default()
    };
    let events = bench
        .step_with(operation(0.001, 12.0, 1), &mut model, &screen)
        .unwrap();
    assert_eq!(model.calls, 1);
    assert_eq!(screen.final_calls.get(), 1);
    assert_eq!(physical(&bench), before);
    veto_journal(&bench, &events);
}

#[test]
fn invalid_second_solve_discards_prior_increment_and_journal() {
    let mut bench = bench();
    let before = format!("{bench:?}");
    let mut model = Model {
        invalid_at: Some(2),
        ..Model::default()
    };
    let screen = Screen::default();
    assert!(bench
        .step_with(operation(0.0001, 12.0, 3), &mut model, &screen)
        .is_err());
    assert_eq!(model.calls, 2);
    assert_eq!(format!("{bench:?}"), before);
    assert_eq!(
        screen.final_calls.get(),
        1,
        "invalid output must not reach final safety"
    );
}

#[test]
fn later_prospective_veto_discards_prior_increment_with_only_veto_journal() {
    let mut bench = bench();
    let before = physical(&bench);
    let mut model = Model::default();
    let screen = Screen {
        prospective_veto_above: Some(0.00015),
        ..Screen::default()
    };
    let events = bench
        .step_with(operation(0.0001, 12.0, 3), &mut model, &screen)
        .unwrap();
    assert_eq!(screen.prospective_calls.get(), 2);
    assert_eq!(model.calls, 1);
    assert_eq!(physical(&bench), before);
    veto_journal(&bench, &events);
}

#[test]
fn failed_second_solve_keeps_only_the_prior_supported_increment() {
    let mut expected = bench();
    expected
        .step_with(
            operation(0.0001, 12.0, 1),
            &mut Model::default(),
            &Screen::default(),
        )
        .unwrap();
    let mut bench = bench();
    let mut model = Model {
        fail_at: Some(2),
        ..Model::default()
    };
    let events = bench
        .step_with(operation(0.0001, 12.0, 3), &mut model, &Screen::default())
        .unwrap();
    assert_eq!(model.calls, 2);
    assert_eq!(physical(&bench), physical(&expected));
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::SolverFailed { .. }))
            .count(),
        1
    );
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::Titrated { steps: 1, .. })));
    assert_eq!(bench.log.len(), 1);
    assert_eq!(bench.log[0].events, events);
}

#[test]
fn discarded_full_trial_final_warning_does_not_leak_into_refined_commit() {
    let mut bench = bench();
    let mut model = Model::default();
    let screen = Screen {
        final_warn: true,
        ..Screen::default()
    };
    let events = bench
        .step_with(operation(0.002, 5.0, 1), &mut model, &screen)
        .unwrap();
    let rules: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::HazardWarning { rule, .. } => Some(rule.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(rules, ["settled-dose-0.001000"]);
    assert_eq!(screen.final_calls.get(), 2);
    assert_eq!(model.calls, 2);
    assert!((bench.vessels[0].moles_of(&SpeciesId::new("NaOH")).0 - 0.001).abs() < 1e-12);
}

#[test]
fn settled_refinement_veto_discards_both_full_and_fractional_trials() {
    let mut bench = bench();
    let before = physical(&bench);
    let mut model = Model::default();
    let screen = Screen {
        refinement_veto: true,
        ..Screen::default()
    };
    let events = bench
        .step_with(operation(0.002, 5.0, 1), &mut model, &screen)
        .unwrap();
    assert_eq!(model.calls, 2);
    assert_eq!(screen.final_calls.get(), 2);
    assert_eq!(physical(&bench), before);
    veto_journal(&bench, &events);
}

#[test]
fn allowed_refinement_commits_only_the_independently_declared_half_dose() {
    let mut bench = bench();
    let mut model = Model::default();
    let events = bench
        .step_with(operation(0.002, 5.0, 1), &mut model, &Screen::default())
        .unwrap();
    assert_eq!(model.calls, 2);
    assert!((bench.vessels[0].moles_of(&SpeciesId::new("NaOH")).0 - 0.001).abs() < 1e-12);
    assert!(events.iter().any(|event| matches!(event, Event::Titrated { endpoint_reached: Some(true), total_volume, .. } if (total_volume.0 - 0.001).abs() < 1e-12)));
    assert_eq!(bench.log.len(), 1);
    assert_eq!(bench.log[0].events, events);
}
