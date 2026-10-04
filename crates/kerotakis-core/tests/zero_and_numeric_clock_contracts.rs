//! Supplementary contracts prompted by frozen fourth-fifty A17/A21/B15.
use kerotakis_core::*;
use std::cell::Cell;
struct Count(Cell<usize>);
impl Equilibrator for Count {
    fn name(&self) -> &'static str {
        "clock-counter"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.0.set(self.0.get() + 1);
        v.temperature = Kelvin(330.0);
        Ok(vec![])
    }
}
fn bench() -> Bench {
    let mut b = Bench::new();
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    b
}
fn physical(b: &Bench) -> serde_json::Value {
    let mut j = serde_json::to_value(b).unwrap();
    j["log"] = serde_json::json!([]);
    j
}
#[test]
fn zero_wait_and_zero_charge_accept_without_physical_or_solver_change() {
    for op in [
        Operator::Wait { seconds: 0.0 },
        Operator::Electrolyse {
            vessel: VesselId(0),
            amps: 0.0,
            seconds: 30.0,
        },
        Operator::Electrolyse {
            vessel: VesselId(0),
            amps: 0.1,
            seconds: 0.0,
        },
    ] {
        let mut b = bench();
        let before = physical(&b);
        let mut s = Count(Cell::new(0));
        b.step_with(op, &mut s, &PermissiveScreen).unwrap();
        assert_eq!(physical(&b), before);
        assert_eq!(s.0.get(), 0);
    }
}
#[test]
fn invalid_direct_wait_is_atomic_and_skips_solver() {
    for seconds in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut b = bench();
        let before = serde_json::to_value(&b).unwrap();
        let mut s = Count(Cell::new(0));
        assert!(b
            .step_with(Operator::Wait { seconds }, &mut s, &PermissiveScreen)
            .is_err());
        assert_eq!(serde_json::to_value(b).unwrap(), before);
        assert_eq!(s.0.get(), 0);
    }
}
#[test]
fn invalid_direct_charge_is_atomic_including_zero_other_operand() {
    for (amps, seconds) in [
        (f64::NAN, 1.0),
        (f64::INFINITY, 1.0),
        (-1.0, 0.0),
        (0.0, f64::INFINITY),
        (1.0, f64::NAN),
        (1.0, -1.0),
    ] {
        let mut b = bench();
        let before = serde_json::to_value(&b).unwrap();
        let mut s = Count(Cell::new(0));
        assert!(b
            .step_with(
                Operator::Electrolyse {
                    vessel: VesselId(0),
                    amps,
                    seconds
                },
                &mut s,
                &PermissiveScreen
            )
            .is_err());
        assert_eq!(serde_json::to_value(b).unwrap(), before);
        assert_eq!(s.0.get(), 0);
    }
}
#[test]
fn cli_grammar_accepts_zero_wait_current_and_duration() {
    for line in [
        "wait 0s",
        "wait 0min",
        "electrolyse v1 0A 30s",
        "electrolyse v1 0.1A 0s",
    ] {
        assert!(script::parse_op(line).unwrap().is_some(), "{line}");
    }
}
#[test]
fn suffixed_clock_and_current_accept_scientific_notation() {
    assert!(
        matches!(script::parse_op("wait 1e-3min").unwrap(),Some(Operator::Wait{seconds}) if (seconds-0.06).abs()<1e-15)
    );
    assert!(
        matches!(script::parse_op("electrolyse v1 1e-3A 2e1s").unwrap(),Some(Operator::Electrolyse{amps,seconds,..}) if amps==0.001 && seconds==20.0)
    );
}
#[test]
fn negative_and_nonfinite_unit_inputs_refuse_before_execution() {
    for line in [
        "wait -1s",
        "wait NaNs",
        "wait 1e999s",
        "electrolyse v1 -1A 30s",
        "electrolyse v1 0A 1e999s",
        "electrolyse v1 1e308A 1e308min",
    ] {
        assert!(script::parse_op(line).is_err(), "{line}");
    }
}
