//! Host-contract regressions, not new chemical-yield experiments.
//! The public Lab tests use the actual shipped stack/oracle and no JS hook.
//! Adversarial writers below test transaction machinery only.
use super::*;
use kerotakis_core::{
    aqueous_continuation::{characterize_optional_via, characterize_via},
    vessel::StepStart,
    Kelvin, Moles, Phase, SolveError, SpeciesId, Vessel, VesselId,
};
use serde_json::Value;
use std::{cell::RefCell, rc::Rc};

fn lab() -> Lab {
    Lab::new().expect("real cache/analytic Lab constructor")
}
fn script(lab: &mut Lab, input: &str) -> Value {
    serde_json::from_str(&lab.run_script(input).expect("canonical valid script")).unwrap()
}
fn events(result: &Value) -> Vec<&Value> {
    result["steps"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|step| step["events"].as_array().unwrap())
        .collect()
}
fn family_reason(result: &Value) -> Option<&Value> {
    events(result).into_iter().find_map(|event| {
        let reason = &event["reason"];
        (event["event"] == "not_yet_modeled"
            && reason["key"] == "not-modeled.family-declined-at-gate"
            && slot(reason, "family")["text"]["text"] == "fischer-esterification")
            .then_some(reason)
    })
}
fn slot<'a>(reason: &'a Value, key: &str) -> &'a Value {
    reason["slots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|pair| pair[0] == key)
        .map(|pair| &pair[1])
        .unwrap()
}
// AmbientClock legitimately redistributes phases (including cold acetic
// acid). Close the complete species ledger rather than freezing phase labels.
fn retained_reagents(before: &Vessel, after: &Vessel) {
    for species in ["CH3COOH", "ethanol"] {
        let key = SpeciesId::new(species);
        close(before.moles_of(&key).0, 0.01);
        close(after.moles_of(&key).0, before.moles_of(&key).0);
        for vessel in [before, after] {
            let mut total = 0.0;
            for phase in [Phase::Solid, Phase::Liquid, Phase::Aqueous, Phase::Gas] {
                let mut phase_total = 0.0;
                for portion in vessel
                    .contents
                    .iter()
                    .filter(|p| p.species == key && p.phase == phase)
                {
                    assert!(portion.moles.0.is_finite() && portion.moles.0 >= 0.0);
                    phase_total += portion.moles.0;
                }
                assert!(phase_total.is_finite() && phase_total >= 0.0);
                total += phase_total;
            }
            close(total, vessel.moles_of(&key).0);
        }
    }
}
fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
}

#[test]
fn real_lab_fischer_wait_forwards_two_actual_cold_temperature_gates() {
    let mut observed = Vec::new();
    for celsius in [15, 25] {
        let mut lab = lab();
        lab.set_locale("de");
        script(&mut lab, &format!("register lv3\nadd v1 CH3COOH 0.01mol @ {celsius}C\nadd v1 ethanol 0.01mol @ {celsius}C\n"));
        let before = lab.bench.vessels[0].clone();
        let result = script(&mut lab, "wait 1s\n");
        let after = &lab.bench.vessels[0];
        close(after.elapsed_seconds - before.elapsed_seconds, 1.0);
        retained_reagents(&before, after);
        assert!(!after
            .contents
            .iter()
            .any(|p| p.species.0 == "ethyl_acetate"));
        assert!(!events(&result).iter().any(|e| e["event"] == "org_reacted"));
        let reason = family_reason(&result).expect("actual shipped oracle's Fischer wait boundary");
        assert_eq!(slot(reason, "gate")["term"]["en"], "temperature_k");
        let detail = &slot(reason, "why")["phrase"]["phrase"];
        assert_eq!(detail["key"], "family-gate.temperature");
        let actual = slot(detail, "actual")["number"]["text"]
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert!((actual - after.temperature.0).abs() < 0.06);
        observed.push(actual);
        let rendered = result["steps"][0]["rendered"].to_string();
        assert!(rendered.contains("Temperatur"));
        assert!(rendered.contains("mindestens"));
        assert!(!rendered.contains("temperature_k"));
    }
    assert_ne!(observed[0], observed[1]);
}

#[test]
fn real_lab_fischer_warm_wait_forwards_real_missing_catalyst_gate() {
    let mut lab = lab();
    lab.set_locale("fr");
    script(
        &mut lab,
        "register lv3\nadd v1 CH3COOH 0.01mol @ 65C\nadd v1 ethanol 0.01mol @ 65C\n",
    );
    let before = lab.bench.vessels[0].clone();
    let result = script(&mut lab, "wait 1s\n");
    let after = &lab.bench.vessels[0];
    close(after.elapsed_seconds - before.elapsed_seconds, 1.0);
    assert!(after.temperature.0 >= 333.15);
    close(after.free_proton, 0.0);
    retained_reagents(&before, after);
    assert!(!after
        .contents
        .iter()
        .any(|p| p.species.0 == "H2SO4" || p.species.0 == "ethyl_acetate"));
    let reason = family_reason(&result).expect("actual shipped missing-catalyst boundary");
    assert_eq!(slot(reason, "gate")["term"]["en"], "catalyst");
    let detail = &slot(reason, "why")["phrase"]["phrase"];
    assert_eq!(detail["key"], "family-gate.catalyst-missing");
    let rendered = result["steps"][0]["rendered"].to_string();
    for expected in ["catalyseur", "acide sulfurique", "ions hydrogène"] {
        assert!(rendered.contains(expected), "{rendered}");
    }
}

#[test]
fn real_lab_unmatched_wait_keeps_clock_and_inventory_without_fischer_gate() {
    for input in ["add v1 water 2mol\n", "add v1 ethanol 0.01mol\n"] {
        let mut lab = lab();
        script(&mut lab, input);
        let before = lab.bench.vessels[0].clone();
        let result = script(&mut lab, "wait 1s\n");
        let after = &lab.bench.vessels[0];
        close(after.elapsed_seconds - before.elapsed_seconds, 1.0);
        assert_eq!(before.contents, after.contents);
        assert!(family_reason(&result).is_none());
    }
}

#[test]
fn real_lab_successful_pressure_reading_retains_direct_model_route() {
    let mut lab = lab();
    script(&mut lab, "seal v1 250mL\n");
    let before = lab.bench.vessels[0].clone();
    assert!(before.gas_moles().0 > 0.0, "real trapped-air headspace");
    let result = script(&mut lab, "measure v1 pressure\n");
    let step = &result["steps"][0];
    let measurement = step["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["event"] == "measured" && event["instrument"] == "pressure_gauge")
        .expect("actual admitted pressure measurement");
    let value = measurement["value"].as_f64().unwrap();
    assert!(value.is_finite() && value > 0.0);
    assert_eq!(before.contents, lab.bench.vessels[0].contents);
    let route = step["routes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|route| route["solver"] == "instrument:pressure-gauge")
        .expect("direct operation's owning route must survive CombinedSolver");
    assert_eq!(route["kind"], "computed");
    assert_eq!(route["chemistry"], false);
    assert_eq!(route["vessel"], 0);
    assert_eq!(route["outcome"]["succeeded"]["event_count"], 1);
}

fn water() -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "readback control");
    vessel.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    vessel
}
#[test]
fn combined_real_analytic_adapter_characterizes_current_water_like_concrete_adapter() {
    let mut concrete = kerotakis_phreeqc::PhreeqcEquilibrator::new().unwrap();
    let mut direct_vessel = water();
    let mut browser = BrowserAqueous {
        inner: &mut concrete,
    };
    let (direct_events, direct_receipt) =
        characterize_optional_via(&mut browser, &mut direct_vessel)
            .expect("real analytic adapter characterizes its actual solvent-only domain");
    assert!(
        direct_receipt.is_none(),
        "solvent-only answer is not a complete receipt"
    );
    assert_eq!(
        direct_vessel.solution.as_ref().unwrap().scope,
        kerotakis_core::SolutionScope::SolventOnly
    );
    let mut aqueous = kerotakis_phreeqc::PhreeqcEquilibrator::new().unwrap();
    let mut stack = SolverStack::new(vec![]);
    let mut combined = CombinedSolver {
        stack: &mut stack,
        aqueous: &mut aqueous,
    };
    let mut actual = water();
    let (events, receipt) = characterize_optional_via(&mut combined, &mut actual)
        .expect("outer wrapper forwards actual optional aqueous characterization");
    assert!(
        receipt.is_none(),
        "wrapper must not upgrade a partial answer"
    );
    assert_eq!(
        actual.solution.as_ref().unwrap().scope,
        kerotakis_core::SolutionScope::SolventOnly
    );
    assert_eq!(format!("{actual:?}"), format!("{direct_vessel:?}"));
    assert_eq!(
        serde_json::to_value(events).unwrap(),
        serde_json::to_value(direct_events).unwrap()
    );
    assert!(actual.solution.as_ref().unwrap().ph.is_finite());
    // Strict complete characterization must retain the genuine partial-state
    // boundary, rather than manufacture a complete receipt from this answer.
    let before = format!("{actual:?}");
    assert!(characterize_via(&mut combined, &mut actual).is_err());
    assert_eq!(format!("{actual:?}"), before);
    let direct_before = format!("{direct_vessel:?}");
    assert!(characterize_via(&mut browser, &mut direct_vessel).is_err());
    assert_eq!(format!("{direct_vessel:?}"), direct_before);
    assert_eq!(
        concrete.engine_calls(),
        0,
        "analytic water, not a fake hook"
    );
    assert_eq!(aqueous.engine_calls(), 0);
}

#[test]
fn combined_refuses_ambiguous_real_aqueous_authorities() {
    let mut aqueous = kerotakis_phreeqc::PhreeqcEquilibrator::new().unwrap();
    let mut stack = SolverStack::new(vec![Box::new(
        kerotakis_phreeqc::PhreeqcEquilibrator::new().unwrap(),
    )]);
    let mut vessel = water();
    let before = format!("{vessel:?}");
    let mut combined = CombinedSolver {
        stack: &mut stack,
        aqueous: &mut aqueous,
    };
    assert!(characterize_via(&mut combined, &mut vessel).is_err());
    assert_eq!(format!("{vessel:?}"), before);
}

struct NoOptIn;
impl Equilibrator for NoOptIn {
    fn name(&self) -> &'static str {
        "no-opt-in-control"
    }
    fn equilibrate(&mut self, _: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        Ok(vec![])
    }
}
#[test]
fn untrusted_wrapper_cannot_get_receipt_from_existing_solution() {
    let mut aqueous = kerotakis_phreeqc::PhreeqcEquilibrator::new().unwrap();
    let mut vessel = water();
    let (_, receipt) = characterize_optional_via(&mut aqueous, &mut vessel).unwrap();
    assert!(receipt.is_none());
    assert_eq!(
        vessel.solution.as_ref().unwrap().scope,
        kerotakis_core::SolutionScope::SolventOnly
    );
    let before = format!("{vessel:?}");
    assert!(characterize_via(&mut NoOptIn, &mut vessel).is_err());
    assert_eq!(before, format!("{vessel:?}"));
}

// Deliberately adversarial acceptance fixtures. None claims chemical yield.
#[derive(Clone, Copy)]
enum Proposal {
    MutateThenErr,
    UnbalancedMaterial,
    InvalidTemperature,
    InvalidGas,
    FiniteGas,
}
struct Writer(Proposal);
impl Equilibrator for Writer {
    fn name(&self) -> &'static str {
        "adversarial-contract-writer"
    }
    fn chemistry_applies(&self, _: &Vessel) -> bool {
        false
    }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        match self.0 {
            Proposal::MutateThenErr => {
                vessel.deposit(SpeciesId::new("NaCl"), Moles(0.25), Phase::Solid);
                vessel.excess_enthalpy_j = 17.0;
                Err(SolveError::NotConverged {
                    reason: None,
                    solver: self.name().into(),
                    detail: "deliberate failed proposal".into(),
                })
            }
            Proposal::UnbalancedMaterial => {
                vessel.deposit(SpeciesId::new("NaCl"), Moles(0.25), Phase::Solid);
                Ok(vec![])
            }
            Proposal::InvalidTemperature => {
                vessel.temperature = Kelvin(-1.0);
                Ok(vec![])
            }
            Proposal::InvalidGas => Ok(vec![Event::GasEvolved {
                vessel: vessel.id,
                species: SpeciesId::new("O2"),
                moles: Moles(-0.001),
            }]),
            Proposal::FiniteGas => {
                let n = vessel.withdraw_phase(&SpeciesId::new("O2"), Moles(0.002), Phase::Gas);
                close(n.0, 0.002);
                Ok(vec![Event::GasEvolved {
                    vessel: vessel.id,
                    species: SpeciesId::new("O2"),
                    moles: n,
                }])
            }
        }
    }
}
#[derive(Default)]
struct Seen {
    calls: usize,
    state: String,
    gas_out: Vec<(SpeciesId, Moles)>,
}
struct Observer(Rc<RefCell<Seen>>);
impl Equilibrator for Observer {
    fn name(&self) -> &'static str {
        "later-contract-observer"
    }
    fn chemistry_applies(&self, _: &Vessel) -> bool {
        false
    }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let mut seen = self.0.borrow_mut();
        seen.calls += 1;
        seen.state = format!("{vessel:?}");
        seen.gas_out = vessel.step_start.as_ref().unwrap().gas_out.clone();
        Ok(vec![])
    }
}
fn transaction_fixture(proposal: Proposal) -> (Vessel, Rc<RefCell<Seen>>, SolverStack) {
    let mut vessel = Vessel::new(VesselId(0), "transaction only");
    if matches!(proposal, Proposal::FiniteGas) {
        vessel.deposit(SpeciesId::new("O2"), Moles(0.002), Phase::Gas);
    }
    vessel.step_start = Some(StepStart::capture(&vessel));
    let seen = Rc::new(RefCell::new(Seen::default()));
    let solvers: Vec<Box<dyn Equilibrator>> =
        vec![Box::new(Writer(proposal)), Box::new(Observer(seen.clone()))];
    let stack = if matches!(proposal, Proposal::UnbalancedMaterial) {
        SolverStack::with_required_conservation(solvers, 1e-10).unwrap()
    } else {
        SolverStack::new(solvers)
    };
    (vessel, seen, stack)
}
fn assert_rejected_proposal_parity(proposal: Proposal) {
    let (mut native, native_seen, mut native_stack) = transaction_fixture(proposal);
    let before = format!("{native:?}");
    let native_events = native_stack.equilibrate(&mut native).unwrap();
    assert_eq!(format!("{native:?}"), before, "native rollback reference");
    assert_eq!(native_seen.borrow().calls, 1);
    assert!(native_events.iter().any(|event| matches!(event, Event::SolverFailed { solver, .. } if solver == "adversarial-contract-writer")));
    let (mut actual, seen, mut stack) = transaction_fixture(proposal);
    let mut aqueous = kerotakis_phreeqc::PhreeqcEquilibrator::new().unwrap();
    let result = CombinedSolver {
        stack: &mut stack,
        aqueous: &mut aqueous,
    }
    .equilibrate(&mut actual);
    assert!(
        result.is_ok(),
        "one rejected child must not silence later independent routes"
    );
    let actual_events = result.unwrap();
    assert_eq!(seen.borrow().calls, 1);
    assert_eq!(
        seen.borrow().state,
        native_seen.borrow().state,
        "later route sees rolled-back state"
    );
    assert_eq!(
        format!("{actual:?}"),
        before,
        "all rejected material/thermal changes roll back"
    );
    assert!(actual_events.iter().any(|event| matches!(event, Event::SolverFailed { solver, .. } if solver == "adversarial-contract-writer")));
    assert!(stack
        .last_routes
        .iter()
        .any(|route| route.solver == "adversarial-contract-writer"
            && matches!(route.outcome, SolverRouteOutcome::Failed)));
}
#[test]
fn combined_mutating_error_rolls_back_before_later_route_like_native_stack() {
    assert_rejected_proposal_parity(Proposal::MutateThenErr);
}
#[test]
fn combined_invalid_finite_temperature_rolls_back_like_native_stack() {
    assert_rejected_proposal_parity(Proposal::InvalidTemperature);
}
#[test]
fn combined_invalid_outward_gas_event_is_rejected_like_native_stack() {
    assert_rejected_proposal_parity(Proposal::InvalidGas);
}
#[test]
fn combined_successful_outward_gas_is_booked_before_later_route_like_native_stack() {
    let (mut native, native_seen, mut native_stack) = transaction_fixture(Proposal::FiniteGas);
    native_stack.equilibrate(&mut native).unwrap();
    let expected = vec![(SpeciesId::new("O2"), Moles(0.002))];
    assert_eq!(native_seen.borrow().gas_out, expected);
    let (mut actual, seen, mut stack) = transaction_fixture(Proposal::FiniteGas);
    let mut aqueous = kerotakis_phreeqc::PhreeqcEquilibrator::new().unwrap();
    CombinedSolver {
        stack: &mut stack,
        aqueous: &mut aqueous,
    }
    .equilibrate(&mut actual)
    .unwrap();
    assert_eq!(seen.borrow().calls, 1);
    assert_eq!(
        seen.borrow().gas_out,
        expected,
        "downstream owner sees actual outward boundary"
    );
    assert_eq!(actual.step_start.as_ref().unwrap().gas_out, expected);
    assert_eq!(actual.contents, native.contents);
}

#[test]
fn combined_installed_routes_preserve_required_material_conservation_policy() {
    assert_rejected_proposal_parity(Proposal::UnbalancedMaterial);
}
