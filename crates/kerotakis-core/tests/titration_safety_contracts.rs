//! Source-informed titration screen contracts, frozen before execution.
use kerotakis_core::ops::Endpoint;
use kerotakis_core::*;
use std::cell::RefCell;
#[derive(Default)]
struct Model {
    calls: usize,
}
impl Equilibrator for Model {
    fn name(&self) -> &'static str {
        "linear-contract-model"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.calls += 1;
        let ph = 3.0 + 800.0 * v.moles_of(&SpeciesId::new("NaOH")).0;
        v.solution = Some(
            serde_json::from_value(serde_json::json!({"ph":ph,"ionic_strength":0.01})).unwrap(),
        );
        Ok(Vec::new())
    }
}
struct Screen {
    mode: u8,
    probes: RefCell<Vec<(Vessel, Vessel)>>,
}
impl Screen {
    fn new(mode: u8) -> Self {
        Self {
            mode,
            probes: RefCell::new(vec![]),
        }
    }
}
impl SafetyScreen for Screen {
    fn assess(&self, _: &Vessel) -> SafetyVerdict {
        panic!("titration must screen the pour with its before state")
    }
    fn assess_pour(&self, before: &Vessel, after: &Vessel) -> SafetyVerdict {
        self.probes
            .borrow_mut()
            .push((before.clone(), after.clone()));
        let n = after.moles_of(&SpeciesId::new("NaOH")).0;
        if self.mode == 1
            || (self.mode == 2 && n > 0.015)
            || (self.mode == 3 && (0.004..0.006).contains(&n))
        {
            SafetyVerdict::Veto {
                reason: "contract dose veto".into(),
            }
        } else if self.mode == 4 {
            SafetyVerdict::Warn {
                severity: Severity::Danger,
                rule: format!("dose-{n:.6}"),
                hazard: "contract warning".into(),
                real_world: "contract".into(),
            }
        } else {
            SafetyVerdict::Allow
        }
    }
}
fn bench() -> Bench {
    let mut b = Bench::new();
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    b.vessels[0].solution =
        Some(serde_json::from_value(serde_json::json!({"ph":3.0,"ionic_strength":0.01})).unwrap());
    b
}
fn op(target: f64, max: u32) -> Operator {
    Operator::Titrate {
        vessel: VesselId(0),
        titrant: SpeciesId::new("NaOH"),
        concentration: 10.0,
        step: Liters(0.001),
        target_ph: target,
        max_steps: max,
        endpoint: Endpoint::Ph,
    }
}
fn physical(b: &Bench) -> serde_json::Value {
    let mut v = serde_json::to_value(b).unwrap();
    v.as_object_mut().unwrap().remove("log");
    v
}
fn journal(b: &Bench, events: &[Event]) {
    assert_eq!(b.log.len(), 1);
    assert_eq!(
        serde_json::to_value(events).unwrap(),
        serde_json::to_value(&b.log[0].events).unwrap()
    );
}
fn only_veto(e: &[Event]) {
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(matches!(&e[0],Event::SafetyVeto{reason} if reason=="contract dose veto"));
}
#[test]
fn first_dose_veto_preserves_complete_bench_without_solver() {
    let mut b = bench();
    let before = physical(&b);
    let mut m = Model::default();
    let s = Screen::new(1);
    let e = b.step_with(op(20.0, 3), &mut m, &s).unwrap();
    assert_eq!(physical(&b), before);
    assert_eq!(m.calls, 0);
    only_veto(&e);
    journal(&b, &e);
}
#[test]
fn later_veto_retains_only_prior_accepted_dose() {
    let mut b = bench();
    let mut m = Model::default();
    let s = Screen::new(2);
    let e = b.step_with(op(30.0, 3), &mut m, &s).unwrap();
    assert_eq!(m.calls, 1);
    assert_eq!(b.vessels[0].moles_of(&SpeciesId::new("NaOH")).0, 0.01);
    assert_eq!(s.probes.borrow().len(), 2);
    assert!(e.iter().any(|e| matches!(e, Event::SafetyVeto { .. })));
    assert!(e.iter().any(|e|matches!(e,Event::Titrated{steps:1,total_volume,endpoint_reached:Some(false),..} if total_volume.0==0.001)));
    journal(&b, &e);
}
#[test]
fn vetoed_refinement_discards_current_full_dose_and_skips_trial_solver() {
    let mut b = bench();
    let before = physical(&b);
    let mut m = Model::default();
    let s = Screen::new(3);
    let e = b.step_with(op(7.0, 1), &mut m, &s).unwrap();
    assert_eq!(physical(&b), before);
    assert_eq!(m.calls, 1);
    assert_eq!(s.probes.borrow().len(), 2);
    only_veto(&e);
    journal(&b, &e);
}
#[test]
fn accepted_fraction_emits_only_its_own_warning() {
    let mut b = bench();
    let mut m = Model::default();
    let s = Screen::new(4);
    let e = b.step_with(op(7.0, 1), &mut m, &s).unwrap();
    let warnings: Vec<_> = e
        .iter()
        .filter_map(|e| {
            if let Event::HazardWarning { rule, .. } = e {
                Some(rule.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(warnings, vec!["dose-0.005000"]);
    assert!(e.iter().any(|e|matches!(e,Event::Titrated{steps:1,total_volume,endpoint_reached:Some(true),..} if total_volume.0==0.0005)));
    journal(&b, &e);
}
#[test]
fn full_dose_probe_has_carrier_water_and_applied_temperature() {
    let mut b = bench();
    b.vessels[0].temperature = Kelvin(323.15);
    let before = b.vessels[0].clone();
    let mut m = Model::default();
    let s = Screen::new(0);
    b.step_with(op(30.0, 1), &mut m, &s).unwrap();
    let probes = s.probes.borrow();
    assert_eq!(probes.len(), 1);
    assert_eq!(
        serde_json::to_value(&probes[0].0).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert_eq!(probes[0].1.temperature, b.vessels[0].temperature);
    assert!(probes[0].1.temperature.0 < 323.15);
    assert_eq!(
        serde_json::to_value(&probes[0].1.contents).unwrap(),
        serde_json::to_value(&b.vessels[0].contents).unwrap()
    );
    assert!(probes[0].1.moles_of(&SpeciesId::new("water")).0 > 2.0);
}
#[test]
fn refinement_probe_starts_from_same_inventory_and_scales_carrier() {
    let mut b = bench();
    let mut m = Model::default();
    let s = Screen::new(0);
    b.step_with(op(7.0, 1), &mut m, &s).unwrap();
    let probes = s.probes.borrow();
    assert_eq!(probes.len(), 2);
    assert_eq!(
        serde_json::to_value(&probes[0].0).unwrap(),
        serde_json::to_value(&probes[1].0).unwrap()
    );
    let water = SpeciesId::new("water");
    let full = probes[0].1.moles_of(&water).0 - 2.0;
    let half = probes[1].1.moles_of(&water).0 - 2.0;
    assert!((half / full - 0.5).abs() < 1e-12);
    assert_eq!(probes[1].1.moles_of(&SpeciesId::new("NaOH")).0, 0.005);
}
#[test]
fn already_reached_endpoint_needs_no_screen_or_solver() {
    let mut b = bench();
    let before = physical(&b);
    let mut m = Model::default();
    let s = Screen::new(1);
    let e = b.step_with(op(3.0, 3), &mut m, &s).unwrap();
    assert_eq!(physical(&b), before);
    assert_eq!(m.calls, 0);
    assert!(s.probes.borrow().is_empty());
    assert!(e.iter().any(|e| matches!(
        e,
        Event::Titrated {
            steps: 0,
            endpoint_reached: Some(true),
            ..
        }
    )));
    journal(&b, &e);
}
#[test]
fn veto_precedes_missing_solver_diagnostic() {
    let mut b = bench();
    let before = physical(&b);
    let s = Screen::new(1);
    let e = b
        .step_with(op(20.0, 3), &mut SolverStack::new(vec![]), &s)
        .unwrap();
    assert_eq!(physical(&b), before);
    only_veto(&e);
    journal(&b, &e);
}
