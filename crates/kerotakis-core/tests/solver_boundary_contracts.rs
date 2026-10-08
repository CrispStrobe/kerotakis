//! Source-informed element balance across explicit gas boundaries.
use kerotakis_core::{
    orchestrator::Orchestrator,
    solve::{Equilibrator, SolveError},
    *,
};
fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
    v
}
fn key(v: &Vessel) -> String {
    format!("{v:?}")
}
#[derive(Clone, Copy)]
enum Boundary {
    Out,
    In,
    Contained,
}
struct Flow {
    boundary: Boundary,
    transfer: f64,
    reported: f64,
    event_vessel: VesselId,
    species: &'static str,
}
impl Equilibrator for Flow {
    fn name(&self) -> &'static str {
        "boundary-flow"
    }
    fn chemistry_applies(&self, _: &Vessel) -> bool {
        true
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        v.honesty_said.push("accepted boundary flow".into());
        match self.boundary {
            Boundary::In => v.deposit(SpeciesId::new("water"), Moles(self.transfer), Phase::Liquid),
            Boundary::Out | Boundary::Contained => {
                v.withdraw_phase(
                    &SpeciesId::new("water"),
                    Moles(self.transfer),
                    Phase::Liquid,
                );
            }
        }
        let vessel = self.event_vessel;
        let species = SpeciesId::new(self.species);
        let moles = Moles(self.reported);
        Ok(vec![match self.boundary {
            Boundary::In => Event::GasAbsorbed {
                vessel,
                species,
                moles,
            },
            Boundary::Out => Event::GasEvolved {
                vessel,
                species,
                moles,
            },
            Boundary::Contained => {
                v.deposit(SpeciesId::new("water"), Moles(self.transfer), Phase::Gas);
                Event::GasContained {
                    vessel,
                    species,
                    moles,
                }
            }
        }])
    }
}
fn flow(boundary: Boundary, transfer: f64, reported: f64) -> Flow {
    Flow {
        boundary,
        transfer,
        reported,
        event_vessel: VesselId(0),
        species: "water",
    }
}
fn refused(flow: Flow) {
    let mut v = water();
    let before = key(&v);
    let events = Orchestrator::new(vec![Box::new(flow)])
        .equilibrate(&mut v)
        .unwrap();
    assert!(matches!(&events[..], [Event::SolverFailed { .. }]));
    assert_eq!(
        key(&v),
        before,
        "refusal must discard derived state and trial events too"
    );
}
#[test]
fn balanced_outlet_commits_owned_state_and_event_together() {
    let mut v = water();
    let events = Orchestrator::new(vec![Box::new(flow(Boundary::Out, 1.0, 1.0))])
        .equilibrate(&mut v)
        .unwrap();
    assert!(matches!(&events[..], [Event::GasEvolved { .. }]));
    assert_eq!(v.moles_of(&SpeciesId::new("water")), Moles(4.0));
    assert_eq!(v.honesty_said, vec!["accepted boundary flow"]);
}
#[test]
fn balanced_inlet_commits_owned_state_and_event_together() {
    let mut v = water();
    let events = Orchestrator::new(vec![Box::new(flow(Boundary::In, 1.0, 1.0))])
        .equilibrate(&mut v)
        .unwrap();
    assert!(matches!(&events[..], [Event::GasAbsorbed { .. }]));
    assert_eq!(v.moles_of(&SpeciesId::new("water")), Moles(6.0));
    assert_eq!(v.honesty_said, vec!["accepted boundary flow"]);
}
#[test]
fn mismatched_inlet_or_outlet_refuses_completely() {
    for boundary in [Boundary::In, Boundary::Out] {
        refused(flow(boundary, 1.0, 0.5));
    }
}
#[test]
fn unsupported_boundary_event_species_refuses() {
    let mut f = flow(Boundary::Out, 0.0, 1.0);
    f.species = "unknown-boundary-gas";
    refused(f);
}
#[test]
fn boundary_event_must_name_its_actual_vessel() {
    let mut f = flow(Boundary::Out, 0.0, 1.0);
    f.event_vessel = VesselId(99);
    refused(f);
}
#[test]
fn invalid_boundary_amounts_refuse_without_trial_events() {
    for boundary in [Boundary::In, Boundary::Out, Boundary::Contained] {
        for n in [f64::NAN, f64::INFINITY, -1.0] {
            refused(flow(boundary, 0.0, n));
        }
    }
}
#[test]
fn contained_gas_remains_owned_without_double_counting_boundary() {
    let mut v = water();
    let events = Orchestrator::new(vec![Box::new(flow(Boundary::Contained, 1.0, 1.0))])
        .equilibrate(&mut v)
        .unwrap();
    assert!(matches!(&events[..], [Event::GasContained { .. }]));
    assert_eq!(v.moles_of(&SpeciesId::new("water")), Moles(5.0));
}
#[test]
fn unaccompanied_finite_boundary_flux_cannot_hide_as_metadata() {
    for boundary in [Boundary::In, Boundary::Out] {
        refused(flow(boundary, 0.0, 1.0));
    }
}
#[test]
fn overflowing_element_boundary_refuses_even_with_finite_event_amount() {
    refused(flow(Boundary::Out, 0.0, f64::MAX));
}
#[test]
fn invalid_conservation_tolerance_refuses_unchanged_or_unbalanced_proposals() {
    for tolerance in [f64::NAN, f64::INFINITY, -1.0] {
        let mut v = water();
        let before = key(&v);
        let mut orchestrator =
            Orchestrator::new(vec![Box::new(flow(Boundary::Contained, 1.0, 1.0))]);
        orchestrator.conservation_tolerance = tolerance;
        assert!(matches!(
            &orchestrator.equilibrate(&mut v).unwrap()[..],
            [Event::SolverFailed { .. }]
        ));
        assert_eq!(key(&v), before);
    }
}
