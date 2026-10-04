//! Source-informed transfer contracts frozen before repairs to Drain/Magnet
//! preparation. No new partition chemistry or compensated vessel adoption.
use std::cell::RefCell;
use kerotakis_core::*;

#[derive(Default)]
struct Capture { proposals: RefCell<Vec<Vessel>>, veto: bool }
impl SafetyScreen for Capture {
    fn assess(&self, vessel: &Vessel) -> SafetyVerdict {
        self.proposals.borrow_mut().push(vessel.clone());
        if self.veto { SafetyVerdict::Veto { reason: "prospective transfer veto".into() } }
        else { SafetyVerdict::Allow }
    }
}
#[derive(Default)]
struct Solver { inputs: Vec<Vessel> }
impl Equilibrator for Solver {
    fn name(&self) -> &'static str { "transfer-input-capture" }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.inputs.push(vessel.clone());
        Ok(Vec::new())
    }
}
fn fixture(kind: &str) -> Bench {
    let mut bench = Bench::new();
    bench.vessels.push(Vessel::new(VesselId(1), "receiver"));
    bench.vessels[0].temperature = Kelvin(340.0);
    bench.vessels[1].temperature = Kelvin(280.0);
    bench.vessels[1].deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    match kind {
        "magnet" => bench.vessels[0].deposit(SpeciesId::new("Fe"), Moles(1.0), Phase::Solid),
        "drain" => {
            bench.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
            bench.vessels[0].deposit(SpeciesId::new("hexane"), Moles(1.0), Phase::Liquid);
            bench.vessels[0].deposit(SpeciesId::new("NaCl"), Moles(0.001), Phase::Aqueous);
        },
        "filter" | "decant" => bench.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid),
        "mix" => {
            bench.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
            bench.vessels.push(Vessel::new(VesselId(2), "mix receiver"));
            bench.vessels[2].temperature = Kelvin(300.0);
            bench.vessels[2].deposit(SpeciesId::new("water"), Moles(0.25), Phase::Liquid);
        },
        _ => panic!("kind"),
    }
    bench
}
fn operation(kind: &str, target: usize) -> Operator {
    match kind {
        "magnet" => Operator::Magnet { from: VesselId(0), to: VesselId(target) },
        "drain" => Operator::Drain { from: VesselId(0), to: VesselId(target) },
        "filter" => Operator::Filter { from: VesselId(0), to: VesselId(target) },
        "decant" => Operator::Decant { from: VesselId(0), to: VesselId(target), fraction: 1.0 },
        "mix" => Operator::Mix { a: VesselId(0), b: VesselId(1), into: VesselId(2), fraction_a: 1.0, fraction_b: 1.0 },
        _ => panic!("kind"),
    }
}
fn physical(bench: &Bench) -> serde_json::Value {
    let mut value = serde_json::to_value(bench).unwrap();
    value.as_object_mut().unwrap().remove("log"); value
}
fn atomic_veto(kind: &str, target: usize) {
    let mut bench = fixture(kind); let before = physical(&bench);
    let mut solver = Solver::default();
    let events = bench.step_with(operation(kind,target), &mut solver, &Capture { veto:true, ..Capture::default() }).unwrap();
    assert_eq!(physical(&bench), before);
    assert!(solver.inputs.is_empty(), "a veto must precede solver execution");
    assert_eq!(events.len(),1);
    assert!(matches!(&events[0], Event::SafetyVeto { reason } if reason=="prospective transfer veto"));
    assert_eq!(bench.log.len(),1);
}
fn receiver_id(kind: &str) -> usize { if kind=="mix" { 2 } else { 1 } }
fn receiver_proposal(kind: &str, bench: &mut Bench) -> Vessel {
    let screen = Capture::default(); let mut solver = Solver::default();
    bench.step_with(operation(kind,1), &mut solver, &screen).unwrap();
    let proposals = screen.proposals.borrow();
    let proposed = proposals.iter().find(|v| v.id==VesselId(receiver_id(kind))).expect("screen the prospective receiver");
    let committed = solver.inputs.iter().find(|v| v.id==VesselId(receiver_id(kind))).expect("receiver solver input");
    assert_eq!(serde_json::to_value(proposed).unwrap(), serde_json::to_value(committed).unwrap(), "screen and solver must see the same raw proposal");
    proposed.clone()
}

#[test] fn magnet_veto_precedes_solver_and_is_atomic() { atomic_veto("magnet",1); }
#[test] fn drain_veto_precedes_solver_and_is_atomic() { atomic_veto("drain",1); }
#[test] fn drain_veto_removes_generated_receiver() { atomic_veto("drain",3); }

#[test]
fn magnetic_solid_carries_sensible_heat_into_adiabatic_receiver() {
    let mut bench=fixture("magnet");
    bench.step_with(operation("magnet",1), &mut Solver::default(), &PermissiveScreen).unwrap();
    assert!(bench.vessels[1].temperature.0>280.0 && bench.vessels[1].temperature.0<340.0);
    assert_eq!(bench.vessels[1].moles_of(&SpeciesId::new("Fe")).0,1.0);
    assert_eq!(bench.vessels[0].moles_of(&SpeciesId::new("Fe")).0,0.0);
}
#[test]
fn magnetic_solid_keeps_thermostatted_receiver_temperature() {
    let mut bench=fixture("magnet"); bench.vessels[1].thermal_mode=ThermalMode::Thermostatted(Kelvin(280.0));
    receiver_proposal("magnet", &mut bench);
    assert_eq!(bench.vessels[1].temperature.0,280.0);
}
#[test]
fn magnetic_proposal_matches_source_temperature_and_heat_metadata_commit() {
    let mut bench=fixture("magnet"); bench.vessels[0].unpriced_heat.push(SpeciesId::new("Fe"));
    let proposal=receiver_proposal("magnet", &mut bench);
    assert!(proposal.unpriced_heat.contains(&SpeciesId::new("Fe")));
    assert!(proposal.temperature.0>280.0);
}
#[test]
fn drain_proposal_matches_actual_lower_layer_and_thermal_commit() {
    let mut bench=fixture("drain");
    let proposal=receiver_proposal("drain", &mut bench);
    assert_eq!(proposal.moles_of(&SpeciesId::new("water")).0,3.0);
    assert_eq!(proposal.moles_of(&SpeciesId::new("NaCl")).0,0.001);
    assert_eq!(proposal.moles_of(&SpeciesId::new("hexane")).0,0.0);
    assert!(proposal.temperature.0>280.0 && proposal.temperature.0<340.0);
}
#[test]
fn filter_shared_receiver_carries_unknown_heat_before_screening() {
    let mut bench=fixture("filter"); bench.vessels[0].unpriced_heat.push(SpeciesId::new("water"));
    let proposal=receiver_proposal("filter", &mut bench);
    assert!(proposal.unpriced_heat.contains(&SpeciesId::new("water")));
}
fn sealed_receiver(kind: &str) {
    let mut bench=fixture(kind);
    let target=receiver_id(kind);
    bench.vessels[target].headspace=Headspace::Sealed { volume: Liters(0.5) };
    bench.vessels[target].deposit(SpeciesId::new("N2"),Moles(0.01),Phase::Gas);
    bench.vessels[target].refresh_pressure();
    let proposal=receiver_proposal(kind,&mut bench);
    let expected=0.01*constants::GAS_CONSTANT*proposal.temperature.0/0.0005;
    assert!((proposal.pressure.0/expected-1.0).abs()<1e-10, "owned gas pressure must follow the raw proposal temperature");
}
#[test] fn magnet_screen_uses_current_raw_sealed_pressure() { sealed_receiver("magnet"); }
#[test] fn drain_screen_uses_current_raw_sealed_pressure() { sealed_receiver("drain"); }
#[test] fn filter_shared_receiver_uses_current_raw_sealed_pressure() { sealed_receiver("filter"); }

fn partitioned_donor_refusal(organic_moles: f64) {
    let mut bench=fixture("drain"); bench.vessels[0].temperature=Kelvin(298.15);
    bench.vessels[0].contents.retain(|p| p.species.0=="water");
    bench.vessels[0].deposit(SpeciesId::new("hexane"),Moles(organic_moles),Phase::Liquid);
    bench.vessels[0].deposit(SpeciesId::new("I2"),Moles(0.001),Phase::Aqueous);
    let before=serde_json::to_value(&bench).unwrap(); let mut solver=Solver::default();
    let error=bench.step_with(operation("drain",1),&mut solver,&PermissiveScreen).expect_err("partial iodine withdrawal must accurately debit its donor");
    assert!(matches!(error,BenchError::TransferDonorPrecision { .. }));
    assert_eq!(serde_json::to_value(bench).unwrap(),before);
    assert!(solver.inputs.is_empty());
}
#[test] fn drain_partition_refuses_a_swallowed_positive_donor_debit() { partitioned_donor_refusal(1e20); }
#[test] fn drain_partition_refuses_a_changing_inaccurate_donor_debit() { partitioned_donor_refusal(1e10); }

#[test]
fn ordinary_partitioned_drain_conserves_the_supported_iodine_transfer() {
    let mut bench=fixture("drain"); bench.vessels[0].temperature=Kelvin(298.15);
    bench.vessels[0].deposit(SpeciesId::new("I2"),Moles(0.001),Phase::Aqueous);
    let events=bench.step_with(operation("drain",1),&mut Solver::default(),&PermissiveScreen).unwrap();
    let retained=bench.vessels[0].moles_of(&SpeciesId::new("I2")).0;
    let moved=bench.vessels[1].moles_of(&SpeciesId::new("I2")).0;
    assert!(retained>0.0 && moved>0.0);
    assert!(((0.001-retained)/moved-1.0).abs()<1e-8);
    assert!(((retained+moved)/0.001-1.0).abs()<1e-12);
    assert!(events.iter().any(|e|matches!(e,Event::Drained { .. })));
}


#[test] fn decant_shared_receiver_uses_current_raw_sealed_pressure() { sealed_receiver("decant"); }
#[test] fn mix_shared_receiver_uses_current_raw_sealed_pressure() { sealed_receiver("mix"); }

#[derive(Default)]
struct Context { pours: RefCell<Vec<(Vessel,Vessel)>> }
impl SafetyScreen for Context {
    fn assess(&self, _: &Vessel) -> SafetyVerdict { panic!("transfer must supply contextual before/after pour") }
    fn assess_pour(&self, before: &Vessel, after: &Vessel) -> SafetyVerdict {
        self.pours.borrow_mut().push((before.clone(),after.clone())); SafetyVerdict::Allow
    }
}
fn contextual(kind: &str) {
    let mut bench=fixture(kind); let target=receiver_id(kind);
    let before=serde_json::to_value(&bench.vessels[target]).unwrap();
    let screen=Context::default(); let mut solver=Solver::default();
    bench.step_with(operation(kind,target),&mut solver,&screen).unwrap();
    let pours=screen.pours.borrow(); assert_eq!(pours.len(),1);
    assert_eq!(serde_json::to_value(&pours[0].0).unwrap(),before);
    let input=solver.inputs.iter().find(|v|v.id==VesselId(target)).unwrap();
    assert_eq!(serde_json::to_value(&pours[0].1).unwrap(),serde_json::to_value(input).unwrap());
}
#[test] fn magnet_uses_contextual_receiver_screen() { contextual("magnet"); }
#[test] fn drain_uses_contextual_receiver_screen() { contextual("drain"); }
#[test] fn filter_uses_contextual_receiver_screen() { contextual("filter"); }
#[test] fn decant_uses_contextual_receiver_screen() { contextual("decant"); }
#[test] fn mix_uses_contextual_receiver_screen() { contextual("mix"); }

fn stale_characterization(kind: &str) {
    let mut bench=fixture(kind); let target=receiver_id(kind);
    bench.vessels[target].solution=Some(serde_json::from_value(serde_json::json!({"ph":2.0,"ionic_strength":0.01})).unwrap());
    let proposal=receiver_proposal(kind,&mut bench);
    assert!(proposal.solution.is_none(),"incoming matter invalidates prior solution before screening");
}
#[test] fn filter_screen_has_no_stale_solution_characterization() { stale_characterization("filter"); }
#[test] fn decant_screen_has_no_stale_solution_characterization() { stale_characterization("decant"); }
#[test] fn mix_screen_has_no_stale_solution_characterization() { stale_characterization("mix"); }

fn carried_heat(kind: &str) {
    let mut bench=fixture(kind); bench.vessels[0].unpriced_heat.push(SpeciesId::new("water"));
    let proposal=receiver_proposal(kind,&mut bench);
    assert!(proposal.unpriced_heat.contains(&SpeciesId::new("water")));
}
#[test] fn decant_screen_includes_carried_unknown_heat() { carried_heat("decant"); }
#[test] fn mix_screen_includes_carried_unknown_heat() { carried_heat("mix"); }
