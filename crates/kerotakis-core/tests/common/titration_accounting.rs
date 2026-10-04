use kerotakis_core::*;
use kerotakis_core::ops::Endpoint;
#[derive(Default)]
pub struct Model { pub calls: usize, pub quantum: bool, pub fail: bool, pub invalid: bool }
impl Equilibrator for Model {
 fn name(&self)->&'static str { "accounting-contract" }
 fn equilibrate(&mut self,v:&mut Vessel)->Result<Vec<Event>,SolveError> {
  self.calls+=1;
  if self.fail { return Err(SolveError::NotConverged{solver:self.name().into(),detail:"contract failure".into()}); }
  let ph=if self.quantum {11.0} else {3.0+800.0*v.moles_of(&SpeciesId::new("NaOH")).0};
  v.solution=Some(serde_json::from_value(serde_json::json!({"ph":ph,"ionic_strength":0.01})).unwrap());
  if self.invalid { v.contents[0].moles=Moles(-1.0); }
  Ok(vec![])
 }
}
pub fn bench()->Bench {
 let mut b=Bench::new();b.vessels[0].deposit(SpeciesId::new("water"),Moles(2.0),Phase::Liquid);
 b.vessels[0].solution=Some(serde_json::from_value(serde_json::json!({"ph":3.0,"ionic_strength":0.01})).unwrap());b
}
pub fn op(concentration:f64,volume:f64,target:f64,max:u32)->Operator {
 Operator::Titrate{vessel:VesselId(0),titrant:SpeciesId::new("NaOH"),concentration,step:Liters(volume),target_ph:target,max_steps:max,endpoint:Endpoint::Ph}
}
pub fn physical(b:&Bench)->serde_json::Value { let mut v=serde_json::to_value(b).unwrap();v.as_object_mut().unwrap().remove("log");v }
pub fn journal(b:&Bench,e:&[Event]) { assert_eq!(b.log.len(),1);assert_eq!(serde_json::to_value(e).unwrap(),serde_json::to_value(&b.log[0].events).unwrap()); }
pub fn water_dose()->f64 { species::lookup(&SpeciesId::new("water")).unwrap().moles_from_liters(Liters(0.001)).0 }
pub fn stock(b:&mut Bench,key:&str,n:f64) { b.stock.stock(key,n,stock::StockUnit::Mole); }
pub fn remaining(b:&Bench,key:&str)->f64 { b.stock.remaining(key).unwrap().amount }
pub fn quantity_refusal(mut b:Bench,op:Operator,mut model:Model,calls:usize) {
 let before=physical(&b);let e=b.step_with(op,&mut model,&PermissiveScreen).unwrap();
 assert_eq!(physical(&b),before);assert_eq!(model.calls,calls);
 assert!(e.iter().any(|e|matches!(e,Event::NotYetModeled{..})),"{e:?}");
 assert!(!e.iter().any(|e|matches!(e,Event::Titrated{..})));journal(&b,&e);
}
