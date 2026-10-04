//! A safety stop must not be described as exhaustion of the burette limit.
use kerotakis_core::ops::Endpoint;
use kerotakis_core::*;
struct Veto;
impl SafetyScreen for Veto {
    fn assess(&self, _: &Vessel) -> SafetyVerdict {
        SafetyVerdict::Veto {
            reason: "stop dose".into(),
        }
    }
}
struct NoSolve;
impl Equilibrator for NoSolve {
    fn name(&self) -> &'static str {
        "must-not-solve"
    }
    fn equilibrate(&mut self, _: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        panic!("vetoed dose reached solver")
    }
}
fn stopped(endpoint: Endpoint) {
    let mut b = Bench::new();
    b.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    b.vessels[0].solution =
        Some(serde_json::from_value(serde_json::json!({"ph":3.0,"ionic_strength":0.01})).unwrap());
    let before = serde_json::to_value(&b.vessels).unwrap();
    let events = b
        .step_with(
            Operator::Titrate {
                vessel: VesselId(0),
                titrant: SpeciesId::new("NaOH"),
                concentration: 10.0,
                step: Liters(0.001),
                target_ph: 7.0,
                max_steps: 9,
                endpoint,
            },
            &mut NoSolve,
            &Veto,
        )
        .unwrap();
    assert_eq!(serde_json::to_value(&b.vessels).unwrap(), before);
    assert_eq!(
        events.len(),
        1,
        "must not claim nine increments were attempted: {events:?}"
    );
    assert!(matches!(&events[0], Event::SafetyVeto { reason } if reason == "stop dose"));
    assert_eq!(b.log.len(), 1);
    assert_eq!(
        serde_json::to_value(&b.log[0].events).unwrap(),
        serde_json::to_value(events).unwrap()
    );
}
#[test]
fn colour_endpoint_veto_does_not_claim_step_limit_exhaustion() {
    stopped(Endpoint::ColourPersists);
}
#[test]
fn potential_endpoint_veto_does_not_claim_undefined_potential_at_step_limit() {
    stopped(Endpoint::Pe {
        compare: ops::Compare::Above,
        value: 8.0,
    });
}
