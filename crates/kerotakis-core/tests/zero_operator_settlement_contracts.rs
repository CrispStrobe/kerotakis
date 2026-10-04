//! Zero work is not a request to advance chemistry or disturb a free surface.
use kerotakis_core::*;
use std::cell::Cell;
struct Mutate(Cell<usize>);
impl Equilibrator for Mutate {
    fn name(&self) -> &'static str {
        "unexpected-settlement"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.0.set(self.0.get() + 1);
        v.temperature = Kelvin(340.0);
        Ok(vec![])
    }
}
struct Screen(Cell<usize>);
impl SafetyScreen for Screen {
    fn assess(&self, _: &Vessel) -> SafetyVerdict {
        self.0.set(self.0.get() + 1);
        SafetyVerdict::Allow
    }
}
fn fixture() -> Bench {
    let mut b = Bench::new();
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    b.vessels.push(Vessel::new(VesselId(1), "beaker"));
    b.vessels.push(Vessel::new(VesselId(2), "beaker"));
    b
}
#[test]
fn all_zero_work_routes_skip_screen_and_solver() {
    let ops = [
        Operator::Heat {
            vessel: VesselId(0),
            energy: Joules(0.0),
            source: None,
        },
        Operator::Cool {
            vessel: VesselId(0),
            energy: Joules(0.0),
        },
        Operator::Decant {
            from: VesselId(0),
            to: VesselId(1),
            fraction: 0.0,
        },
        Operator::Mix {
            a: VesselId(0),
            b: VesselId(1),
            into: VesselId(2),
            fraction_a: 0.0,
            fraction_b: 0.0,
        },
        Operator::Evaporate {
            vessel: VesselId(0),
            fraction: 0.0,
        },
        Operator::Distil {
            from: VesselId(0),
            to: VesselId(1),
            fraction: Some(0.0),
            energy: None,
            stages: 1,
        },
    ];
    for op in ops {
        let mut b = fixture();
        let mut before = serde_json::to_value(&b).unwrap();
        let mut solver = Mutate(Cell::new(0));
        let screen = Screen(Cell::new(0));
        let description = format!("{op:?}");
        b.step_with(op, &mut solver, &screen).unwrap();
        let mut after = serde_json::to_value(b).unwrap();
        before["log"] = serde_json::json!([]);
        after["log"] = serde_json::json!([]);
        assert_eq!(after, before, "{description}");
        assert_eq!(solver.0.get(), 0, "{description}");
        assert_eq!(screen.0.get(), 0, "{description}");
    }
}
#[test]
fn zero_pour_and_zero_still_do_not_create_receiver() {
    for op in [
        Operator::Decant {
            from: VesselId(0),
            to: VesselId(50),
            fraction: 0.0,
        },
        Operator::Distil {
            from: VesselId(0),
            to: VesselId(50),
            fraction: Some(0.0),
            energy: None,
            stages: 1,
        },
    ] {
        let mut b = fixture();
        let mut s = Mutate(Cell::new(0));
        b.step_with(op, &mut s, &PermissiveScreen).unwrap();
        assert_eq!(b.vessels.len(), 3);
        assert_eq!(s.0.get(), 0);
    }
}
