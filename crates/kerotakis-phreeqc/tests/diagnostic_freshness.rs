//! Cached readings do not certify a new medium or survive their owner's failure.
//! Warm-up is actual native chemistry. Failures below are explicitly injected
//! transaction faults, not claimed measurements of chemical convergence.
#![cfg(feature = "engine")]

use kerotakis_core::aqueous_continuation::TrustedAqueousReadback;
use kerotakis_core::instrument::{InstrumentContract, PhMeter};
use kerotakis_core::*;
use kerotakis_org::family_oracle::family_equilibrator;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn native_seed() -> (Vessel, PhreeqcEquilibrator) {
    let mut eq = PhreeqcEquilibrator::new().expect("native engine");
    let mut v = Vessel::new(VesselId(0), "native seed");
    v.temperature = Kelvin(335.0);
    v.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    v.deposit(SpeciesId::new("NaCl"), Moles(0.01), Phase::Solid);
    eq.equilibrate(&mut v).expect("actual aqueous warm-up");
    assert!(eq.engine_calls() > 0);
    assert!(v.solution.as_ref().is_some_and(|s| s.ph.is_finite()));
    assert!(PhMeter.applies(&v));
    v.resolved.solution = v.solution.clone();
    v.resolved.valid = true;
    (v, eq)
}

fn family_feed(seed: &Vessel, water: f64, ethanol: f64) -> Vessel {
    let mut v = seed.clone();
    // Deliberately retain the real earlier reading to challenge admission.
    v.contents.clear();
    v.deposit(SpeciesId::new("water"), Moles(water), Phase::Liquid);
    v.deposit(SpeciesId::new("ethanol"), Moles(ethanol), Phase::Liquid);
    v.deposit(SpeciesId::new("ethyl_acetate"), Moles(0.001), Phase::Liquid);
    v.deposit(SpeciesId::new("NaOH"), Moles(0.001), Phase::Solid);
    v
}

#[test]
fn a_real_cached_solution_cannot_override_the_current_family_medium() {
    let (seed, _) = native_seed();
    let mut router = family_equilibrator();
    for (water, ethanol) in [(0.001, 0.2), (0.0, 0.2), (0.0, 0.0)] {
        let mut v = family_feed(&seed, water, ethanol);
        let before = v.contents.clone();
        let evaluation = router.evaluate(&v);
        assert!(evaluation
            .ready
            .iter()
            .all(|r| r.family != "alkaline-ester-hydrolysis"));
        assert!(
            evaluation
                .declined
                .iter()
                .any(|r| r.family == "alkaline-ester-hydrolysis" && r.gate == "medium"),
            "{evaluation:?}"
        );
        let events = router.equilibrate(&mut v).expect("explicit medium decline");
        assert!(!events.iter().any(
            |e| matches!(e, Event::OrgReacted { name, .. } if name == "alkaline-ester-hydrolysis")
        ));
        assert_eq!(v.contents, before);
    }
    let mut positive = family_feed(&seed, 5.55, 0.001);
    let events = router
        .equilibrate(&mut positive)
        .expect("water-majority positive control");
    assert!(
        events.iter().any(
            |e| matches!(e, Event::OrgReacted { name, .. } if name == "alkaline-ester-hydrolysis")
        ),
        "{events:?}"
    );
    assert!(positive.moles_of(&SpeciesId::new("ethyl_acetate")).0 < 1e-9);
}

struct InjectedFailure {
    aqueous_owner: Option<PhreeqcEquilibrator>,
}

impl Equilibrator for InjectedFailure {
    fn name(&self) -> &'static str {
        "injected-transaction-failure"
    }
    fn trusted_aqueous_readback(&mut self) -> Option<&mut dyn TrustedAqueousReadback> {
        self.aqueous_owner
            .as_mut()
            .map(|eq| eq as &mut dyn TrustedAqueousReadback)
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        // A failed proposal may have touched owned matter before discovering
        // an error. Neither the proposal nor stale owned diagnostics may leak.
        v.contents.clear();
        v.temperature = Kelvin(999.0);
        Err(SolveError::NotConverged {
            reason: None,
            solver: self.name().into(),
            detail: "explicitly injected transaction failure".into(),
        })
    }
}

fn assert_withdrawn(v: &Vessel) {
    assert!(
        v.solution.is_none(),
        "failed aqueous owner's previous reading survived"
    );
    assert_eq!((v.free_proton, v.free_hydroxide), (0.0, 0.0));
    assert!(!v.resolved.valid && v.resolved.solution.is_none());
    assert!(!PhMeter.applies(v));
}

#[test]
fn failed_aqueous_stack_rolls_back_matter_and_withdraws_owned_readings() {
    let (mut v, eq) = native_seed();
    let before = v.clone();
    let mut stack = SolverStack::new(vec![Box::new(InjectedFailure {
        aqueous_owner: Some(eq),
    })]);
    let events = stack
        .equilibrate(&mut v)
        .expect("failure remains a structured event");
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::SolverFailed { .. })));
    assert_eq!(v.contents, before.contents);
    assert_eq!(v.temperature, before.temperature);
    assert_withdrawn(&v);
}

#[test]
fn failed_aqueous_phase_coupling_rolls_back_matter_and_withdraws_owned_readings() {
    let (mut v, eq) = native_seed();
    let before = v.clone();
    let mut failure = InjectedFailure {
        aqueous_owner: Some(eq),
    };
    let events = kerotakis_core::solve::equilibrate_phase_coupled(&mut failure, &mut v)
        .expect("structured failure");
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::SolverFailed { .. })));
    assert_eq!(v.contents, before.contents);
    assert_eq!(v.temperature, before.temperature);
    assert_withdrawn(&v);
}

#[test]
fn an_unrelated_failed_solver_does_not_withdraw_valid_aqueous_output() {
    for phase_coupled in [false, true] {
        let (mut v, _) = native_seed();
        let before = v.clone();
        let mut failure = InjectedFailure {
            aqueous_owner: None,
        };
        let events = if phase_coupled {
            kerotakis_core::solve::equilibrate_phase_coupled(&mut failure, &mut v)
                .expect("structured failure")
        } else {
            SolverStack::new(vec![Box::new(failure)])
                .equilibrate(&mut v)
                .expect("structured failure")
        };
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })));
        assert_eq!(v.contents, before.contents);
        assert_eq!(v.temperature, before.temperature);
        assert_eq!(v.solution, before.solution);
        assert_eq!(v.resolved.solution, before.resolved.solution);
        assert_eq!(v.resolved.valid, before.resolved.valid);
        assert!(PhMeter.applies(&v));
    }
}
