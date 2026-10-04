//! Correct the recorded pure-water extraction fixture without rewriting its forecast.
use kerotakis_core::*;
use std::cell::Cell;
struct Screen(Cell<usize>);
impl SafetyScreen for Screen {
    fn assess(&self, _: &Vessel) -> SafetyVerdict {
        self.0.set(self.0.get() + 1);
        SafetyVerdict::Veto {
            reason: "supported extraction veto".into(),
        }
    }
}
struct NoSettlement;
impl Equilibrator for NoSettlement {
    fn name(&self) -> &'static str {
        "no-settlement"
    }
    fn equilibrate(&mut self, _: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        panic!("a refused extraction must not settle")
    }
}
fn run(supported: bool) {
    let mut b = Bench::new();
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    if supported {
        b.vessels[0].deposit(SpeciesId::new("I2"), Moles(0.001), Phase::Aqueous);
    }
    let before = serde_json::to_value(&b).unwrap();
    let screen = Screen(Cell::new(0));
    let events = b
        .step_with(
            Operator::Extract {
                from: VesselId(0),
                to: VesselId(1),
                solvent: SpeciesId::new("hexane"),
                total_solvent: Moles(0.5),
                stages: 1,
            },
            &mut NoSettlement,
            &screen,
        )
        .unwrap();
    let mut after = serde_json::to_value(&b).unwrap();
    after["log"] = before["log"].clone();
    assert_eq!(after, before);
    assert_eq!(b.log.len(), 1);
    assert_eq!(
        serde_json::to_value(&events).unwrap(),
        serde_json::to_value(&b.log[0].events).unwrap()
    );
    if supported {
        assert_eq!(screen.0.get(), 1);
        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0], Event::SafetyVeto { reason } if reason == "supported extraction veto")
        );
    } else {
        assert_eq!(screen.0.get(), 0);
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })));
        assert!(!events.iter().any(|e| matches!(e, Event::SafetyVeto { .. })));
    }
}
#[test]
fn supported_extraction_veto_is_atomic_before_receiver_creation() {
    run(true);
}
#[test]
fn pure_water_extraction_refuses_before_safety_screen() {
    run(false);
}
