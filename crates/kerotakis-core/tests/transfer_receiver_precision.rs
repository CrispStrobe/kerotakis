//! Receiver increments must retain the complete transfer, or restore the entire bench.
use kerotakis_core::*;

#[derive(Default)]
struct Counter(usize);
impl Equilibrator for Counter {
    fn name(&self) -> &'static str { "receiver-counter" }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.0 += 1;
        vessel.temperature.0 += 1.0;
        Ok(Vec::new())
    }
}
fn fixture(species: &str, phase: Phase, amount: f64, bulk: f64) -> Bench {
    let mut b = Bench::new();
    b.vessels[0].deposit(SpeciesId::new(species), Moles(amount), phase);
    b.vessels.push(Vessel::new(VesselId(1), "receiver"));
    b.vessels[1].deposit(SpeciesId::new(species), Moles(bulk), phase);
    b
}
fn refused(mut b: Bench, op: Operator) {
    let before = serde_json::to_value(&b).unwrap();
    let mut solver = Counter::default();
    let result = b.step_with(op, &mut solver, &PermissiveScreen);
    assert!(result.is_err(), "unrepresentable transfer succeeded: {result:?}");
    assert_eq!(serde_json::to_value(&b).unwrap(), before);
    assert_eq!(solver.0, 0);
}
fn decant() -> Operator {
    Operator::Decant { from: VesselId(0), to: VesselId(1), fraction: 1.0 }
}
#[test]
fn decant_refuses_lost_and_inaccurate_receiver_increments() {
    for amount in [1e-20, 0.75 * f64::EPSILON] {
        refused(fixture("water", Phase::Liquid, amount, 1.0), decant());
    }
}
#[test]
fn filter_refuses_lost_and_inaccurate_receiver_increments() {
    for amount in [1e-20, 0.75 * f64::EPSILON] {
        refused(fixture("NaCl", Phase::Aqueous, amount, 1.0),
            Operator::Filter { from: VesselId(0), to: VesselId(1) });
    }
}
#[test]
fn magnet_refuses_lost_and_inaccurate_receiver_increments() {
    for amount in [1e-20, 0.75 * f64::EPSILON] {
        refused(fixture("Fe", Phase::Solid, amount, 1.0),
            Operator::Magnet { from: VesselId(0), to: VesselId(1) });
    }
}
#[test]
fn mixing_checks_second_stream_against_first_stream() {
    let mut b = fixture("water", Phase::Liquid, 1.0, 1e-20);
    b.vessels.push(Vessel::new(VesselId(2), "receiver"));
    refused(b, Operator::Mix { a: VesselId(0), b: VesselId(1), into: VesselId(2),
        fraction_a: 1.0, fraction_b: 1.0 });
}
#[test]
fn drain_refuses_trace_solute_loss_after_moving_water() {
    let mut b = fixture("NaCl", Phase::Aqueous, 1e-20, 1.0);
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    b.vessels[0].deposit(SpeciesId::new("hexane"), Moles(1.0), Phase::Liquid);
    refused(b, Operator::Drain { from: VesselId(0), to: VesselId(1) });
}
#[test]
fn condensed_phase_merge_cannot_hide_a_transfer() {
    let mut b = fixture("water", Phase::Liquid, 1e-20, 0.0);
    b.vessels[1].deposit(SpeciesId::new("water"), Moles(1.0), Phase::Aqueous);
    refused(b, decant());
}
#[test]
fn failed_autocreated_receiver_is_removed() {
    let mut b = fixture("water", Phase::Liquid, 1.0, 0.0);
    b.vessels.pop();
    b.vessels[0].contents.push(vessel::Portion {
        species: SpeciesId::new("water"), moles: Moles(1e-20), phase: Phase::Liquid });
    refused(b, decant());
}
#[test]
fn empty_receivers_retain_positive_traces_and_ordinary_bulk_additions_work() {
    for amount in [1e-100, 1e-20, 0.25] {
        let mut b = fixture("water", Phase::Liquid, amount, 0.0);
        b.step_with(decant(), &mut SolverStack::new(vec![]), &PermissiveScreen).unwrap();
        assert_eq!(b.vessels[1].moles_of(&SpeciesId::new("water")).0, amount);
    }
    let mut b = fixture("water", Phase::Liquid, 0.25, 1.0);
    b.step_with(decant(), &mut SolverStack::new(vec![]), &PermissiveScreen).unwrap();
    assert_eq!(b.vessels[1].moles_of(&SpeciesId::new("water")).0, 1.25);
}
