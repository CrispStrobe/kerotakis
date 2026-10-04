//! Preserve physical free-surface geometry while zero work skips all processing.
use kerotakis_core::*;
use std::cell::Cell;
struct Calls(Cell<usize>);
impl SafetyScreen for Calls {
    fn assess(&self, _: &Vessel) -> SafetyVerdict {
        self.0.set(self.0.get() + 1);
        SafetyVerdict::Allow
    }
}
struct Solver(Cell<usize>);
impl Equilibrator for Solver {
    fn name(&self) -> &'static str {
        "zero-transfer-observer"
    }
    fn equilibrate(&mut self, _: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.0.set(self.0.get() + 1);
        Ok(vec![])
    }
}
#[test]
fn zero_transfers_preserve_coloured_receiver_and_all_owned_state() {
    for mix in [false, true] {
        let mut b = Bench::new();
        for line in [
            "add v1 water 2mol",
            "new",
            "add v2 whole_milk 10mL",
            "add v2 food_colour_red 1mL",
            "new",
            "add v3 water 1mol",
        ] {
            b.step(script::parse_op(line).unwrap().unwrap()).unwrap();
        }
        assert!(!b.vessels[1].surface_colours.is_empty());
        b.vessels[0].temperature = Kelvin(323.15);
        b.vessels[2].temperature = Kelvin(303.15);
        b.vessels[1].thermal_mode = vessel::ThermalMode::Adiabatic;
        let before = serde_json::to_value(&b.vessels).unwrap();
        let op = if mix {
            Operator::Mix {
                a: VesselId(0),
                b: VesselId(2),
                into: VesselId(1),
                fraction_a: 0.0,
                fraction_b: 0.0,
            }
        } else {
            Operator::Decant {
                from: VesselId(0),
                to: VesselId(1),
                fraction: 0.0,
            }
        };
        let screen = Calls(Cell::new(0));
        let mut solver = Solver(Cell::new(0));
        b.step_with(op, &mut solver, &screen).unwrap();
        assert_eq!(serde_json::to_value(&b.vessels).unwrap(), before);
        assert_eq!(screen.0.get(), 0);
        assert_eq!(solver.0.get(), 0);
    }
}
