//! Corrections and supplementary controls after the frozen extreme-solvent
//! forecasts reached the modeled single-phase domain, not a donor transfer.
use kerotakis_core::*;

#[derive(Default)] struct Counter(usize);
impl Equilibrator for Counter {
    fn name(&self)->&'static str { "drain-partition-counter" }
    fn equilibrate(&mut self,_:&mut Vessel)->Result<Vec<Event>,SolveError> { self.0+=1;Ok(vec![]) }
}
fn fixture(hexane:f64,iodine:f64)->Bench {
    let mut b=Bench::new();b.vessels.push(Vessel::new(VesselId(1),"receiver"));
    b.vessels[0].deposit(SpeciesId::new("water"),Moles(2.0),Phase::Liquid);
    b.vessels[0].deposit(SpeciesId::new("hexane"),Moles(hexane),Phase::Liquid);
    b.vessels[0].deposit(SpeciesId::new("I2"),Moles(iodine),Phase::Aqueous);b
}
fn operation()->Operator { Operator::Drain { from:VesselId(0),to:VesselId(1) } }
fn physical(b:&Bench)->serde_json::Value { let mut x=serde_json::to_value(b).unwrap();x.as_object_mut().unwrap().remove("log");x }
fn quantized_refusal(bits:u64) {
    let mut b=fixture(1.0,f64::from_bits(bits));
    assert!(solve::layered_pair(&b.vessels[0]).is_some(),"fixture must actually be in the modeled two-layer domain");
    let before=serde_json::to_value(&b).unwrap();let mut counter=Counter::default();
    let error=b.step_with(operation(),&mut counter,&PermissiveScreen).expect_err("positive partition yield must be accurately represented");
    assert!(matches!(error,BenchError::TransferDonorPrecision { .. }));
    assert_eq!(serde_json::to_value(b).unwrap(),before);assert_eq!(counter.0,0);
}
#[test] fn modeled_two_layer_drain_refuses_underflowed_positive_partition_yield() { quantized_refusal(1); }
#[test] fn modeled_two_layer_drain_refuses_nonzero_quantized_partition_yield() { quantized_refusal(512); }

fn single_phase_control(hexane:f64) {
    let mut b=fixture(hexane,0.001);
    assert!(solve::layered_pair(&b.vessels[0]).is_none(),"original extreme-ratio forecast does not reach partition transfer");
    let before=physical(&b);let events=b.step_with(operation(),&mut Counter::default(),&PermissiveScreen).unwrap();
    assert_eq!(physical(&b),before);
    assert!(events.iter().any(|e|matches!(e,Event::NotYetModeled { cause:ops::NotModelledCause::NothingToActOn,reason:Some(reason),.. } if reason.key=="not-modeled.drain-a-single-phase-liquid")));
    assert!(!events.iter().any(|e|matches!(e,Event::Drained { .. })));
}
#[test] fn changing_debit_original_fixture_is_matched_by_single_phase_control() { single_phase_control(1e10); }
#[test] fn swallowed_debit_original_fixture_is_matched_by_single_phase_control() { single_phase_control(1e20); }
#[test]
fn ordinary_two_layer_partition_conserves_actual_donor_and_receiver_amounts() {
    let mut b=fixture(1.0,0.001);assert!(solve::layered_pair(&b.vessels[0]).is_some());
    let events=b.step_with(operation(),&mut Counter::default(),&PermissiveScreen).unwrap();
    let retained=b.vessels[0].moles_of(&SpeciesId::new("I2")).0;let moved=b.vessels[1].moles_of(&SpeciesId::new("I2")).0;
    assert!(retained>0.0 && moved>0.0);assert!(((0.001-retained)/moved-1.0).abs()<1e-8);
    assert!(((retained+moved)/0.001-1.0).abs()<1e-12);assert!(events.iter().any(|e|matches!(e,Event::Drained { .. })));
}
