//! Frozen source-informed admission controls; no aqueous chemistry forecast.
use kerotakis_core::ops::Endpoint;
use kerotakis_core::solve::{Equilibrator, SafetyScreen, SafetyVerdict, SolveError};
use kerotakis_core::{Bench, Event, Liters, Moles, Operator, Phase, SpeciesId, Vessel, VesselId};
use std::cell::Cell;

#[derive(Default)]
struct Observer {
    applies: Cell<usize>,
    solves: usize,
}
impl Equilibrator for Observer {
    fn name(&self) -> &'static str {
        "titration-admission-observer"
    }
    fn applies(&self, _: &Vessel) -> bool {
        self.applies.set(self.applies.get() + 1);
        true
    }
    fn equilibrate(&mut self, _: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.solves += 1;
        Ok(Vec::new())
    }
}
#[derive(Default)]
struct Screen {
    calls: Cell<usize>,
    veto: bool,
}
impl SafetyScreen for Screen {
    fn assess(&self, _: &Vessel) -> SafetyVerdict {
        self.calls.set(self.calls.get() + 1);
        if self.veto {
            SafetyVerdict::Veto {
                reason: "frozen trial veto".into(),
            }
        } else {
            SafetyVerdict::Allow
        }
    }
}
fn bench() -> Bench {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    bench
}
fn op(concentration: f64, step: f64, target_ph: f64, max_steps: u32) -> Operator {
    Operator::Titrate {
        vessel: VesselId(0),
        titrant: SpeciesId::new("NaOH"),
        concentration,
        step: Liters(step),
        target_ph,
        max_steps,
        endpoint: Endpoint::Ph,
    }
}
fn physical_state(bench: &Bench) -> serde_json::Value {
    let mut state = serde_json::to_value(bench).unwrap();
    state.as_object_mut().unwrap().remove("log");
    state
}
fn refused_before_hooks(mut bench: Bench, operation: Operator) {
    let before = format!("{bench:?}");
    let mut observer = Observer::default();
    let screen = Screen::default();
    let result = bench.step_with(operation, &mut observer, &screen);
    assert!(
        result.is_err(),
        "malformed admission must return an error: {result:?}"
    );
    assert_eq!(
        format!("{bench:?}"),
        before,
        "state, stock and history must survive refusal"
    );
    assert_eq!(
        observer.applies.get(),
        0,
        "invalid candidates must not reach applicability hooks"
    );
    assert_eq!(
        observer.solves, 0,
        "invalid candidates must not reach a solver"
    );
    assert_eq!(
        screen.calls.get(),
        0,
        "invalid candidates must not reach safety hooks"
    );
}
#[test]
fn nonfinite_concentration_is_refused_before_trial_hooks() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        refused_before_hooks(bench(), op(value, 0.001, 7.0, 1));
    }
}
#[test]
fn nonfinite_step_volume_is_refused_before_trial_hooks() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        refused_before_hooks(bench(), op(0.1, value, 7.0, 1));
    }
}
#[test]
fn nonfinite_ph_target_is_refused_before_trial_hooks() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        refused_before_hooks(bench(), op(0.1, 0.001, value, 1));
    }
}
#[test]
fn nonpositive_inputs_cannot_cancel_each_other_into_a_positive_dose() {
    for (concentration, step) in [
        (0.0, 0.001),
        (-0.1, 0.001),
        (0.1, 0.0),
        (0.1, -0.001),
        (-0.1, -0.001),
    ] {
        refused_before_hooks(bench(), op(concentration, step, 7.0, 1));
    }
}
#[test]
fn overflowing_dose_product_is_refused_before_trial_hooks() {
    refused_before_hooks(bench(), op(f64::MAX, 2.0, 7.0, 1));
}
#[test]
fn finite_dose_inputs_cannot_admit_overflowing_carrier_inventory() {
    // Both factors and their product are finite; the carrier-water amount
    // implied by this volume exceeds the candidate's aggregate numeric domain.
    refused_before_hooks(bench(), op(1e-307, 1e306, 7.0, 1));
}
#[test]
fn invalid_initial_inventory_is_refused_before_trial_hooks() {
    let mut malformed = bench();
    malformed.vessels[0].contents[0].moles = Moles(-1.0);
    refused_before_hooks(malformed, op(0.1, 0.001, 7.0, 1));
}
#[test]
fn ordinary_single_increment_reaches_hooks_and_preserves_the_requested_dose() {
    let mut bench = bench();
    let mut observer = Observer::default();
    let screen = Screen::default();
    bench
        .step_with(op(0.1, 0.001, 7.0, 1), &mut observer, &screen)
        .unwrap();
    assert!(
        screen.calls.get() > 0,
        "prospective valid doses must be screened"
    );
    assert!(observer.applies.get() > 0);
    assert_eq!(observer.solves, 1);
    assert!((bench.vessels[0].moles_of(&SpeciesId::new("NaOH")).0 - 0.0001).abs() < 1e-12);
    assert_eq!(bench.log.len(), 1);
}
#[test]
fn valid_zero_increment_limit_is_a_logged_noop_without_hooks() {
    let mut bench = bench();
    let before = physical_state(&bench);
    let mut observer = Observer::default();
    let screen = Screen::default();
    bench
        .step_with(op(0.1, 0.001, 7.0, 0), &mut observer, &screen)
        .unwrap();
    assert_eq!(physical_state(&bench), before);
    assert_eq!(screen.calls.get(), 0);
    assert_eq!(observer.applies.get(), 0);
    assert_eq!(observer.solves, 0);
    assert_eq!(bench.log.len(), 1);
}
#[test]
fn vetoed_trial_preserves_physical_state_and_records_only_the_veto() {
    let mut bench = bench();
    let before = physical_state(&bench);
    let mut observer = Observer::default();
    let screen = Screen {
        calls: Cell::new(0),
        veto: true,
    };
    let events = bench
        .step_with(op(0.1, 0.001, 7.0, 1), &mut observer, &screen)
        .unwrap();
    assert_eq!(physical_state(&bench), before);
    assert!(screen.calls.get() > 0);
    assert_eq!(observer.applies.get(), 0);
    assert_eq!(observer.solves, 0);
    assert!(matches!(events.as_slice(), [Event::SafetyVeto { .. }]));
    assert_eq!(bench.log.len(), 1);
    assert_eq!(bench.log[0].events, events);
}
