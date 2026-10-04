//! Supplementary reviewed recovery hook contract; no separate old-source baseline.
use kerotakis_core::authority::SpillDestination;
use kerotakis_core::spill::SpillCompartment;
use kerotakis_core::*;
use std::cell::Cell;
#[test]
fn recovery_uses_pour_hook_with_actual_before_and_proposed_after() {
    struct Hook {
        calls: Cell<usize>,
    }
    impl SafetyScreen for Hook {
        fn assess(&self, _: &Vessel) -> SafetyVerdict {
            panic!("use the pour hook")
        }
        fn assess_pour(&self, before: &Vessel, after: &Vessel) -> SafetyVerdict {
            self.calls.set(self.calls.get() + 1);
            assert_eq!(before.contents[0].moles.0, 1.0);
            assert_eq!(after.contents[0].moles.0, 1.5);
            SafetyVerdict::Veto {
                reason: "incoming recovery veto".into(),
            }
        }
    }
    struct NoSolver;
    impl Equilibrator for NoSolver {
        fn name(&self) -> &'static str {
            "no-recovery-settlement"
        }
        fn equilibrate(&mut self, _: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            panic!("a veto must not settle")
        }
    }
    let mut b = Bench::new();
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    let destination = SpillDestination::Tray {
        tray: "hook".into(),
    };
    let mut spill = SpillCompartment::new(destination.clone(), Kelvin::STANDARD);
    spill.contents = b.vessels[0].contents.clone();
    b.spills.push(spill);
    let before = serde_json::to_value(&b).unwrap();
    let hook = Hook {
        calls: Cell::new(0),
    };
    let events = b
        .step_with(
            Operator::RecoverSpill {
                destination,
                to: VesselId(0),
                fraction: 0.5,
            },
            &mut NoSolver,
            &hook,
        )
        .unwrap();
    let mut after = serde_json::to_value(&b).unwrap();
    after["log"] = before["log"].clone();
    assert_eq!(after, before);
    assert_eq!(hook.calls.get(), 1);
    assert!(matches!(&events[..], [Event::SafetyVeto { .. }]));
}
