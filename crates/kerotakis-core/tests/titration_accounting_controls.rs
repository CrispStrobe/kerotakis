//! Supplementary refusal-path and nominal-product controls frozen before repair execution.
mod common {
    include!("common/titration_accounting.rs");
}
use common::*;
use kerotakis_core::*;
#[derive(Default)]
struct Counter(usize);
impl Equilibrator for Counter {
    fn name(&self) -> &'static str {
        "stock-refusal-counter"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.0 += 1;
        v.temperature.0 += 1.0;
        Ok(vec![])
    }
}
fn stock_refused(line: &str, key: &str) {
    let mut b = bench();
    stock(&mut b, key, 0.0);
    let before = physical(&b);
    let mut m = Counter::default();
    let e = b
        .step_with(
            script::parse_op(line).unwrap().unwrap(),
            &mut m,
            &PermissiveScreen,
        )
        .unwrap();
    assert_eq!(physical(&b), before);
    assert_eq!(m.0, 0);
    assert!(e.iter().any(|e| matches!(e, Event::StockExhausted { .. })));
    journal(&b, &e);
}
#[test]
fn ordinary_species_stock_refusal_does_not_settle() {
    stock_refused("add v1 NaOH 0.1mol", "NaOH");
}
#[test]
fn ordinary_material_stock_refusal_does_not_settle() {
    stock_refused("add v1 olive_oil 1g", "olive_oil");
}
fn input_refused(concentration: f64) {
    let mut b = bench();
    let before = physical(&b);
    let mut m = Model::default();
    assert!(b
        .step_with(
            op(concentration, f64::from_bits(1), 30.0, 1),
            &mut m,
            &PermissiveScreen
        )
        .is_err());
    assert_eq!(physical(&b), before);
    assert_eq!(m.calls, 0);
    assert!(b.log.is_empty());
}
#[test]
fn inaccurately_rounded_nominal_titrant_product_refuses_before_trial() {
    input_refused(1.5);
}
#[test]
fn inaccurately_rounded_nominal_carrier_conversion_refuses_before_trial() {
    input_refused(1.0);
}
