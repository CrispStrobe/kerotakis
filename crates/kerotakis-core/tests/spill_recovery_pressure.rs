//! Supplementary gas recovery screening contract frozen during patch review.
use kerotakis_core::authority::SpillDestination;
use kerotakis_core::spill::SpillCompartment;
use kerotakis_core::*;
use std::cell::Cell;
#[test]
fn screened_sealed_receiver_pressure_includes_incoming_gas() {
    struct Pressure(Cell<f64>);
    impl SafetyScreen for Pressure {
        fn assess(&self, v: &Vessel) -> SafetyVerdict {
            self.0.set(v.pressure.0);
            if v.pressure.0 > 200_000.0 {
                SafetyVerdict::Veto {
                    reason: "gas recovery pressure".into(),
                }
            } else {
                SafetyVerdict::Allow
            }
        }
    }
    let mut b = Bench::new();
    b.vessels[0].headspace = Headspace::Sealed {
        volume: Liters(1.0),
    };
    b.vessels[0].refresh_pressure();
    let destination = SpillDestination::Tray { tray: "gas".into() };
    let mut spill = SpillCompartment::new(destination.clone(), Kelvin::STANDARD);
    let mut gas = Vessel::new(VesselId(1), "gas");
    gas.deposit(SpeciesId::new("N2"), Moles(1.0), Phase::Gas);
    spill.contents = gas.contents;
    b.spills.push(spill);
    let before = serde_json::to_value(&b).unwrap();
    let screen = Pressure(Cell::new(0.0));
    let events = b
        .step_with(
            Operator::RecoverSpill {
                destination,
                to: VesselId(0),
                fraction: 1.0,
            },
            &mut SolverStack::new(vec![]),
            &screen,
        )
        .unwrap();
    let mut after = serde_json::to_value(&b).unwrap();
    after["log"] = before["log"].clone();
    assert!(screen.0.get() > 200_000.0);
    assert_eq!(after, before);
    assert!(matches!(&events[..], [Event::SafetyVeto { .. }]));
}
